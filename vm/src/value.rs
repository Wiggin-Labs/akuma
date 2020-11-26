#![allow(non_upper_case_globals, non_snake_case)]

use {get_head, set_head, Environment, Operation};
use self::heap_repr::*;

use string_interner::{get_value, Symbol};

use std::{fmt, ops};

pub enum VType {
    Void = 0,
    Nil = 1,
    Bool = 2,
    Integer = 3,
    Float = 4,
    Symbol = 5,
    Lambda = 6,
    Pair = 7,
    Vec = 8,
    String = 9,
}

impl From<u64> for VType {
    fn from(p: u64) -> VType {
        if p == VType::String as u64 {
            VType::String
        } else if p == VType::Vec as u64 {
            VType::Vec
        } else if p == VType::Lambda as u64 {
            VType::Lambda
        } else if p == VType::Pair as u64 {
            VType::Pair
        } else if p == VType::Void as u64 {
            VType::Void
        } else {
            unreachable!()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct Value(pub u64);

// A signaling NAN constant
// The left-most bit of the significand must be 0, and at least one of the bottom 51 bits must be 1
// in order to differentiate from INF/-INF. We need the bottom 48 bits for pointers, which
// currently only use 48bits on amd64. This leaves us with 4 unused bits (48, 49, 50, and 63). Note
// that the sign bit does not matter, so we *could* use it as part of the tag.
const NAN: u64 = 0x7FF0000000000000;
const TAG_MASK: u64 = 0b111 << 48;
const IMMEDIATE_MASK: u64 = 0b1111 << 44;

// The following values need 32 bits or less, so they all share a tag of 0b000 and use some of the
// folowing bits to differentiate. This lets us pack many types under one tag.
const IMMEDIATE_TAG: u64 = 0b000 << 48;
const VOID_TAG: u64 =   0b0001 << 44;
const NIL_TAG: u64 =    0b0010 << 44;
const BOOL_TAG: u64 =   0b0011 << 44;
const INT_TAG: u64 =    0b0100 << 44;
const SYMBOL_TAG: u64 = 0b0101 << 44;
const TRUE: u64 = 1;
const FALSE: u64 = 0;

const LAMBDA_TAG: u64 = 0b001 << 48;
const PAIR_TAG: u64 =   0b010 << 48;
const VEC_TAG: u64 =    0b011 << 48;
const STRING_TAG: u64 = 0b100 << 48;


//const OTHER_TAG: u64 = 0b101 << 48;
//const OTHER_TAG: u64 = 0b110 << 48;
//const OTHER_TAG: u64 = 0b111 << 48;

// TODO: replace middle & with && when it is allowed in const fn
macro_rules! is_imm {
    ($name:ident, $tag:ident) => {
        pub const fn $name(self) -> bool {
            ((self.0 & NAN) == NAN) & ((self.0 & TAG_MASK) == IMMEDIATE_TAG)
                & ((self.0 & IMMEDIATE_MASK) == $tag)
        }
    };
}

/*
macro_rules! create_pointer {
    ($name:ident, $tag:ident) => {
        pub const fn $name(p: u64) -> Self {
            // We & the pointer with (2^48) - 1 because while Amd64 currently only uses the lower
            // 48bits for pointers, it requires the high 16 bits to be the same as the 48th bit.
            // For our |'s to work properly, we need the upper bits to be 0.
            Value(NAN | $tag | (p & ((1 << 48) - 1)))
        }
    };
}
*/

// TODO: replace middle & with && when it is allowed in const fn
macro_rules! is_pointer {
    ($name:ident, $tag:ident) => {
        pub const fn $name(self) -> bool {
            ((self.0 & NAN) == NAN) & ((self.0 & TAG_MASK) == $tag)
        }
    };
}

macro_rules! to_pointer {
    ($name:ident, $t:ident) => {
        pub fn $name(self) -> Box<$t> {
            let pointer = self.to_pointer();
            unsafe { Box::from_raw(pointer as *mut $t) }
        }
    };
}

impl Value {
    pub fn to_type(self) -> VType {
        if self.is_void() {
            VType::Void
        } else if self.is_nil() {
            VType::Nil
        } else if self.is_bool() {
            VType::Bool
        } else if self.is_integer() {
            VType::Integer
        } else if self.is_float() {
            VType::Float
        } else if self.is_symbol() {
            VType::Symbol
        } else if self.is_lambda() {
            VType::Lambda
        } else if self.is_pair() {
            VType::Pair
        } else if self.is_vec() {
            VType::Vec
        } else if self.is_string() {
            VType::String
        } else {
            unreachable!();
        }
    }

    pub const Void: Self = Value(NAN | VOID_TAG);
    is_imm!(is_void, VOID_TAG);

    pub const Nil: Self = Value(NAN | NIL_TAG);
    is_imm!(is_nil, NIL_TAG);

    // TODO: make const when if is allowed
    pub fn Bool(b: bool) -> Self {
        if b { Self::True } else { Self::False }
    }
    is_imm!(is_bool, BOOL_TAG);

    pub const True: Self = Value(NAN | BOOL_TAG | TRUE);
    // TODO: make const when if is allowed
    pub fn is_true(self) -> bool {
        !self.is_false()
    }

    pub const False: Self = Value(NAN | BOOL_TAG | FALSE);
    // TODO: make const when if is allowed
    pub fn is_false(self) -> bool {
        self.is_bool() && ((self.0 & TRUE) == FALSE)
    }

    pub const fn Integer(i: i32) -> Self {
        Value(NAN | INT_TAG | (i as u32 as u64))
    }
    is_imm!(is_integer, INT_TAG);

    pub const fn to_integer(self) -> i32 {
        self.0 as u32 as i32
    }

    // TODO: make this const when const mem::transmute is stable
    pub fn Float(f: f64) -> Self {
        Value(f.to_bits())
    }

    pub const fn is_float(self) -> bool {
        (self.0 & NAN) != NAN
    }

    // TODO: make this const when const mem::transmute is stable
    pub fn to_float(self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn Symbol(s: Symbol) -> Self {
        // TODO: this should probably check that s is only 32/48 bits
        Value(NAN | SYMBOL_TAG | (*s as u64))
    }
    is_imm!(is_symbol, SYMBOL_TAG);

    pub fn to_symbol(self) -> Symbol {
        Symbol::new(self.0 as u32 as usize)
    }

    pub fn Lambda(env: Environment, code: Vec<Operation>) -> Self {
        let next = get_head();
        let lambda = Box::into_raw(Box::new(Lambda::new(next, env, code)));
        let p = lambda as u64;
        set_head(p, VType::Lambda);
        Value(NAN | LAMBDA_TAG | (p & ((1 << 48) - 1)))
    }
    is_pointer!(is_lambda, LAMBDA_TAG);
    to_pointer!(to_lambda, Lambda);

    pub fn Pair(car: Value, cdr: Value) -> Self {
        let next = get_head();
        let pair = Box::into_raw(Box::new(Pair::new(next, car, cdr)));
        let p = pair as u64;
        set_head(p, VType::Pair);
        Value(NAN | PAIR_TAG | (p & ((1 << 48) - 1)))
    }
    is_pointer!(is_pair, PAIR_TAG);
    to_pointer!(to_pair, Pair);

    pub fn car(self) -> Self {
        let p = self.to_pair();
        let c = p.car;
        Box::into_raw(p);
        c
    }

    pub fn cdr(self) -> Self {
        let p = self.to_pair();
        let c = p.cdr;
        Box::into_raw(p);
        c
    }

    pub fn set_car(self, v: Self) {
        let mut p = self.to_pair();
        p.car = v;
        Box::into_raw(p);
    }

    pub fn set_cdr(self, v: Self) {
        let mut p = self.to_pair();
        p.cdr = v;
        Box::into_raw(p);
    }

    pub fn Vec(v: Vec<Value>) -> Self {
        let next = get_head();
        let vec = Box::into_raw(Box::new(SVec::new(next, v)));
        let p = vec as u64;
        set_head(p, VType::Vec);
        Value(NAN | VEC_TAG | (p & ((1 << 48) - 1)))
    }
    is_pointer!(is_vec, VEC_TAG);
    to_pointer!(to_vec, SVec);

    pub fn String(s: String) -> Self {
        let next = get_head();
        let str = Box::into_raw(Box::new(SString::new(next, s)));
        let p = str as u64;
        set_head(p, VType::String);
        Value(NAN | STRING_TAG | (p & ((1 << 48) - 1)))
    }
    is_pointer!(is_string, STRING_TAG);
    to_pointer!(to_string, SString);

    // TODO: make const when if is allowed
    pub fn to_pointer(self) -> u64 {
        // Amd64 currently only uses the lower 48 bits for pointers, which is what makes NANboxing
        // possible. However, it requires that the upper 16 bits of a pointer be the same as the
        // 48th bit, so here we check whether it is 1 or 0 and set them appropriately.
        if 1 == (self.0 >> 47) & 1 {
            self.0 | (0xFFFF << 48)
        } else {
            self.0 & ((1 << 48) - 1)
        }
    }

    // TODO: the recursion will probably blow the stack for lists.
    pub(crate) fn mark(self) {
        match self.to_type() {
            VType::Lambda => {
                let mut p = self.to_lambda();
                p.gc = p.gc | 1;
                Box::into_raw(p);
            }
            VType::Pair => {
                let mut p = self.to_pair();
                p.gc = p.gc | 1;
                p.car.mark();
                p.cdr.mark();
                Box::into_raw(p);
            }
            VType::Vec => {
                let mut p = self.to_vec();
                for v in &p.vec {
                    v.mark();
                }
                p.gc = p.gc | 1;
                Box::into_raw(p);
            }
            VType::String => {
                let mut p = self.to_string();
                p.gc = p.gc | 1;
                Box::into_raw(p);
            }
            _ => (),
        }
    }

    pub(crate) fn set_gc(p: u64, gc: u64) {
        let ty = VType::from(p >> 56);
        let ptr = if (p >> 55) & 1 == 1 {
            p & 0xFF_FF_FF_FF_FF_FF_FF_FE
        } else {
            p & 0x00_00_FF_FF_FF_FF_FF_FE
        };

        match ty {
            VType::Lambda => {
                let mut p = unsafe { Box::from_raw(ptr as *mut Lambda) };
                p.gc = gc;
                Box::into_raw(p);
            }
            VType::Pair => {
                let mut p = unsafe { Box::from_raw(ptr as *mut Pair) };
                p.gc = gc;
                Box::into_raw(p);
            }
            VType::String => {
                let mut p = unsafe { Box::from_raw(ptr as *mut SString) };
                p.gc = gc;
                Box::into_raw(p);
            }
            VType::Vec => {
                let mut p = unsafe { Box::from_raw(ptr as *mut SVec) };
                p.gc = gc;
                Box::into_raw(p);
            }
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_float() {
            write!(f, "{}", self.to_float())
        } else if self.is_integer() {
            write!(f, "{}", self.to_integer())
        } else if self.is_symbol() {
            let s = self.to_symbol();
            write!(f, "{}", get_value(s).unwrap())
        } else if *self == Value::True {
            write!(f, "#t")
        } else if *self == Value::False {
            write!(f, "#f")
        } else if self.is_nil() {
            write!(f, "()")
        } else if self.is_void() {
            Ok(())
        } else if self.is_lambda() {
            write!(f, "#<procedure>")
        } else if self.is_pair() {
            let p = Value::to_pair(*self);

            write!(f, "({}", p.car)?;
            let mut c = p.cdr;
            while c.is_pair() {
                let p = Value::to_pair(c);
                write!(f, " {}", p.car)?;
                c = p.cdr;
                Box::into_raw(p);
            }
            let r = if c.is_nil() {
                write!(f, ")")
            } else {
                write!(f, " . {})", c)
            };

            Box::into_raw(p);
            r
        } else if self.is_string() {
            let s = Value::to_string(*self);
            let r = write!(f, "\"{}\"", s.str);
            Box::into_raw(s);
            r
        } else if self.is_vec() {
            let vec = Value::to_vec(*self);
            write!(f, "#(")?;
            for (i, v) in vec.vec.iter().enumerate() {
                if i+1 != vec.vec.len() {
                    write!(f, "{}, ", v)?;
                } else {
                    write!(f, "{}", v)?;
                }
            }
            Box::into_raw(vec);
            write!(f, ")")
        } else {
            write!(f, "debug: {:?}", self)
        }
    }
}

impl ops::Deref for Value {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value(v)
    }
}

pub mod heap_repr {
    use super::Value;
    use {Environment, Operation};

    pub struct Lambda {
        pub(crate) gc: u64,
        pub env: Environment,
        pub code: Vec<Operation>,
    }

    impl Lambda {
        pub fn new(root: u64, env: Environment, code: Vec<Operation>) -> Self {
            Lambda {
                gc: root & 0xff_ffff_ffff_ffff,
                env: env,
                code,
            }
        }
    }

    pub struct Pair {
        pub(crate) gc: u64,
        pub car: Value,
        pub cdr: Value,
    }

    impl Pair {
        pub fn new(gc: u64, car: Value, cdr: Value) -> Self {
            Pair {
                // set top byte to 0 so it can be used for gc
                //gc: root & 0xff_ffff_ffff_ffff,
                gc: gc,
                car,
                cdr,
            }
        }
    }

    pub struct SString {
        pub(crate) gc: u64,
        pub str: String,
    }

    impl SString {
        pub fn new(gc: u64, s: String) -> Self {
            SString {
                gc: gc,
                str: s,
            }
        }
    }

    pub struct SVec {
        pub(crate) gc: u64,
        pub vec: Vec<Value>,
    }

    impl SVec {
        pub fn new(gc: u64, v: Vec<Value>) -> Self {
            SVec {
                gc: gc,
                vec: v,
            }
        }
    }
}

use parser::Token;
use ramp::Int;
use ramp::rational::Rational;

use regex::Regex;

use std::f64;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq, PartialOrd, is_enum_variant)]
pub enum Number {
    Exact(ComplexExact),
    Floating(ComplexFloating),
}

impl Number {
    pub fn from_token(t: &Token) -> Self {
        match t {
            Token::ComplexExact(real, imaginary) => {
                let real = if let Some(real) = real {
                    real.parse().unwrap()
                } else {
                    Rational::new(Int::zero(), Int::one())
                };
                let imaginary = if let Some(imaginary) = imaginary {
                    if imaginary.len() == 1{
                        if imaginary == "-" {
                            -Rational::new(Int::one(), Int::one())
                        } else {
                            Rational::new(Int::one(), Int::one())
                        }
                    } else {
                        imaginary.parse().unwrap()
                    }
                } else {
                    Rational::new(Int::zero(), Int::one())
                };
                Number::Exact(ComplexExact::new(real, imaginary))
            }
            Token::ComplexFloating(real, imaginary) => {
                const _RAT: &'static str = r"[+-]?\d+/\d+";
                lazy_static! {
                    static ref RATIONAL: Regex = Regex::new(_RAT).unwrap();
                }

                let real = if let Some(real) = real {
                    if RATIONAL.is_match(real) {
                        Self::rat_to_f64(real)
                    } else {
                        real.parse().unwrap()
                    }
                } else {
                    0f64
                };
                let imaginary = if let Some(imaginary) = imaginary {
                    if imaginary.len() == 1{
                        if imaginary == "-" {
                            -1.0
                        } else {
                            1.0
                        }
                    } else if RATIONAL.is_match(imaginary) {
                        Self::rat_to_f64(imaginary)
                    } else {
                        imaginary.parse().unwrap()
                    }
                } else {
                    0f64
                };
                Number::Floating(ComplexFloating::new(real, imaginary))
            }
            _ => unreachable!(),
        }
    }

    fn rat_to_f64(buf: &str) -> f64 {
        let i = buf.find('/').unwrap();
        let (numerator, denominator) = buf.split_at(i);
        let denominator = denominator.trim_left_matches('/');
        numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap()
    }

    pub fn zero() -> Self {
        Number::Exact(ComplexExact::new(rat_zero(), rat_zero()))
    }

    pub fn one() -> Self {
        Number::Exact(ComplexExact::new(rat_one(), rat_zero()))
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Number::Exact(n) => if !n.is_complex() {
                write!(f, "{}", n.real)
            } else {
                write!(f, "{}+{}i", n.real, n.imaginary)
            },
            Number::Floating(n) => if !n.is_complex() {
                write!(f, "{}", n.real)
            } else {
                write!(f, "{}+{}i", n.real, n.imaginary)
            },
        }
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        match (self, other) {
            (Number::Exact(a), Number::Exact(b)) => {
                Number::Exact(a + b)
            }
            (Number::Exact(a), Number::Floating(b)) => {
                Number::Floating(a.to_floating() + b)
            }
            (Number::Floating(a), Number::Exact(b)) => {
                Number::Floating(a + b.to_floating())
            }
            (Number::Floating(a), Number::Floating(b)) => {
                Number::Floating(a + b)
            }
        }
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Number {
        match self {
            Number::Exact(n) => Number::Exact(-n),
            Number::Floating(n) => Number::Floating(-n),
        }
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        match (self, other) {
            (Number::Exact(a), Number::Exact(b)) => {
                Number::Exact(a - b)
            }
            (Number::Exact(a), Number::Floating(b)) => {
                Number::Floating(a.to_floating() - b)
            }
            (Number::Floating(a), Number::Exact(b)) => {
                Number::Floating(a - b.to_floating())
            }
            (Number::Floating(a), Number::Floating(b)) => {
                Number::Floating(a - b)
            }
        }
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        match (self, other) {
            (Number::Exact(a), Number::Exact(b)) => {
                Number::Exact(a * b)
            }
            (Number::Exact(a), Number::Floating(b)) => {
                Number::Floating(a.to_floating() * b)
            }
            (Number::Floating(a), Number::Exact(b)) => {
                Number::Floating(a * b.to_floating())
            }
            (Number::Floating(a), Number::Floating(b)) => {
                Number::Floating(a * b)
            }
        }
    }
}

impl From<ComplexExact> for Number {
    fn from(n: ComplexExact) -> Number {
        Number::Exact(n)
    }
}

impl From<ComplexFloating> for Number {
    fn from(n: ComplexFloating) -> Number {
        Number::Floating(n)
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Number {
        let numerator = Rational::new(Int::from(n), Int::one());
        Number::Exact(ComplexExact::new(numerator, rat_zero()))
    }
}

impl From<Rational> for Number {
    fn from(n: Rational) -> Number {
        Number::Exact(ComplexExact::new(n, rat_zero()))
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Number {
        Number::Floating(ComplexFloating::new(n, 0f64))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ComplexExact {
    real: Rational,
    imaginary: Rational,
}

impl ComplexExact {
    pub fn new(real: Rational, imaginary: Rational) -> Self {
        ComplexExact {
            real,
            imaginary,
        }
    }

    pub fn is_complex(&self) -> bool {
        self.imaginary != rat_zero()
    }

    pub(crate) fn as_usize(&self) -> usize {
        usize::from(&self.real.clone().into_parts().0)
    }

    pub fn to_floating(self) -> ComplexFloating {
        let real = self.real.to_f64();
        let imaginary = self.imaginary.to_f64();
        ComplexFloating::new(real, imaginary)
    }
}

impl Add for ComplexExact {
    type Output = ComplexExact;

    fn add(self, other: ComplexExact) -> ComplexExact {
        ComplexExact::new(self.real + other.real,
                          self.imaginary + other.imaginary)
    }
}

impl Neg for ComplexExact {
    type Output = ComplexExact;

    fn neg(self) -> ComplexExact {
        ComplexExact {
            real: -self.real,
            imaginary: -self.imaginary
        }
    }
}

impl Sub for ComplexExact {
    type Output = ComplexExact;

    fn sub(self, other: ComplexExact) -> ComplexExact {
        ComplexExact::new(self.real - other.real,
                          self.imaginary - other.imaginary)
    }
}

impl Mul for ComplexExact {
    type Output = ComplexExact;

    fn mul(self, other: ComplexExact) -> ComplexExact {
        let real = (self.real.clone() * other.real.clone())
                   - (self.imaginary.clone() * other.imaginary.clone());
        let imaginary = (self.real * other.imaginary) + (self.imaginary * other.real);
        ComplexExact::new(real, imaginary)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ComplexFloating {
    real: f64,
    imaginary: f64,
}

impl ComplexFloating {
    pub fn new(real: f64, imaginary: f64) -> Self {
        ComplexFloating {
            real,
            imaginary,
        }
    }

    pub fn is_complex(&self) -> bool {
        self.imaginary != 0f64
    }
}

impl Add for ComplexFloating {
    type Output = ComplexFloating;

    fn add(self, other: ComplexFloating) -> ComplexFloating {
        ComplexFloating::new(self.real + other.real,
                          self.imaginary + other.imaginary)
    }
}

impl Neg for ComplexFloating {
    type Output = ComplexFloating;

    fn neg(self) -> ComplexFloating {
        ComplexFloating {
            real: -self.real,
            imaginary: -self.imaginary
        }
    }
}

impl Sub for ComplexFloating {
    type Output = ComplexFloating;

    fn sub(self, other: ComplexFloating) -> ComplexFloating {
        ComplexFloating::new(self.real - other.real,
                             self.imaginary - other.imaginary)
    }
}

impl Mul for ComplexFloating {
    type Output = ComplexFloating;

    fn mul(self, other: ComplexFloating) -> ComplexFloating {
        let real = (self.real * other.real) - (self.imaginary * other.imaginary);
        let imaginary = (self.real * other.imaginary) + (self.imaginary * other.real);
        ComplexFloating::new(real, imaginary)
    }
}

#[inline]
fn rat_zero() -> Rational {
    Rational::new(Int::zero(), Int::one())
}

#[inline]
fn rat_one() -> Rational {
    Rational::new(Int::one(), Int::one())
}

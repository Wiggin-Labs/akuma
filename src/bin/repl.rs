extern crate akuma;
#[cfg(feature="profile")]
extern crate flame;

use std::io::{stdin, stdout, Write};

fn main() {
    let env = akuma::init_env();
    loop {
        // Print prompt
        print!(">> ");
        stdout().flush().unwrap();
        // Read input
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        while input.chars().filter(|&c| c == '(').count()
            > input.chars().filter(|&c| c == ')').count()
        {
            stdin().read_line(&mut input).unwrap();
        }

        if "exit\n" == input {
            break;
        }

        let tokens = match akuma::Parser::parse(&input) {
            Ok(t) => t,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };
        let objects = match akuma::Token::build_ast(tokens) {
            Ok(o) => o,
            Err(e) => {
                println!("ERROR: {}", e);
                continue;
            }
        };

        for object in objects {
            println!("{}", akuma::eval(object, &env));
        }
    }

    #[cfg(feature="profile")]
    flame::dump_html(&mut std::fs::File::create("profile2.html").unwrap()).unwrap();
}

use std::env::args;
use std::fs::read_to_string;
use std::path::*;
use vc_parser::lexer::lex;

fn main() {
    let mut args = args();
    let executable_name = args.next().unwrap();
    if let Some(cockpit_path) = args.next() {
        let tokens = lex(&read_to_string(cockpit_path).unwrap());
        for token in tokens.unwrap() {
            println!("{:?}", token);
        }
    } else {
        println!("Usage: {} <cockpit.ini>", executable_name);
    }
}

use std::env::args;
use std::fs::read_to_string;
use std::path::*;
use vc_parser::lexer::lex;

fn main() {
    let mut args = args();
    let executable_name = args.next().unwrap();
    if let Some(cockpit_path) = args.next() {
        let cockpit_dir = std::path::Path::new(&cockpit_path).parent().unwrap();
        println!(
            "Parsing cockpit file: {} in {}",
            cockpit_path,
            cockpit_dir.display()
        );
        let tokens = lex(std::path::Path::new(&cockpit_path).to_path_buf());
        for token in tokens.unwrap() {
            println!("{:?}", token);
        }
    } else {
        println!("Usage: {} <cockpit.ini>", executable_name);
    }
}

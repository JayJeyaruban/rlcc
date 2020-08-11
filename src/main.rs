#[macro_use]
extern crate lazy_static;

mod tokenizer;

use std::env;
use std::process::exit;
use std::fs;

use tokenizer::parse_tokens;

fn main() {
    let filename = env::args().nth(1);
    if filename == None {
        // Throw some error
        exit(1);
    }

    let file_contents = match filename {
        None => exit(1),
        Some(f) => fs::read_to_string(&f)
    }.expect("Unable to read provided file.");

    println!("{:?}",  parse_tokens(&file_contents));
}

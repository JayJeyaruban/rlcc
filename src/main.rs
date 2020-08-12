#[macro_use]
extern crate lazy_static;

mod parser;
mod tokenizer;

use std::env;
use std::fs;
use std::process::exit;

use parser::parse_tokens;
use tokenizer::tokenize_file;

fn main() {
    let filename = env::args().nth(1);
    if filename == None {
        // Throw some error
        exit(1);
    }

    let file_contents = match filename {
        None => exit(1),
        Some(f) => fs::read_to_string(&f),
    }
    .expect("Unable to read provided file.");

    parse_tokens(&tokenize_file(&file_contents));
}

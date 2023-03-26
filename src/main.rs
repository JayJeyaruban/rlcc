mod parser;
mod tokenizer;

use parser::process_tokens;
use std::{fs, process::ExitCode};
use tokenizer::{ParsedToken, TokenParseResult};

use crate::tokenizer::parse_tokens;

fn main() -> Result<ExitCode, Vec<String>> {
    // let filename = "tests/res/lci/test/1.3-Tests/1-Structure/1-EmptyMainBlock/test.lol";
    let filename = "tests/res/lci/test/1.3-Tests/1-Structure/2-MustBeginWithHAI/test.lol";

    let file_contents = fs::read_to_string(&filename).unwrap();

    let parse_results = parse_tokens(file_contents);

    let (tokens, errs) = split_errs(parse_results);

    let mut errs = errs;

    for err in process_tokens(tokens) {
        errs.push(err);
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        println!("Compilation successful");
        Ok(ExitCode::SUCCESS)
    }
}

fn split_errs(results: Vec<TokenParseResult>) -> (Vec<ParsedToken>, Vec<String>) {
    let mut errs = Vec::new();
    let mut tokens = Vec::new();
    for res in results {
        match res {
            Ok(token) => tokens.push(token),
            Err(err) => errs.push(err),
        }
    }

    (tokens, errs)
}

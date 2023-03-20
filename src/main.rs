mod tokenizer;

use std::fs;
use tokenizer::{KeywordToken, ParsedToken, TokenParseResult};

use crate::tokenizer::parse_tokens;

fn main() -> Result<(), &'static str> {
    let filename = "tests/res/lci/test/1.3-Tests/1-Structure/1-EmptyMainBlock/test.lol";

    let file_contents = fs::read_to_string(&filename).unwrap();

    let parse_results = parse_tokens(file_contents);

    let (tokens, errs) = split_errs(parse_results);

    let tokens = extract_dec(tokens);
    println!("{:?}", &tokens);

    if errs.len() > 0 {
        println!("{:?}", errs);
        Err("There are multiple errors.")
    } else {
        Ok(())
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

fn extract_dec(mut tokens: Vec<ParsedToken>) -> Vec<ParsedToken> {
    let mut hai_i: Option<usize> = None;
    let mut version: Option<usize> = None;
    let mut hai_end: Option<usize> = None;
    for token_i in 0..tokens.len() {
        let token = tokens.get(token_i).unwrap();
        if token == &ParsedToken::Space {
            continue;
        }

        if token == &ParsedToken::Keyword(KeywordToken::Hai) {
            if hai_i.is_some() {
                panic!("Unexpected additional HAI");
            }
            hai_i = Some(token_i);
            continue;
        }

        if token == &ParsedToken::NewLine {
            if hai_i.is_none() && version.is_none() {
                continue;
            } else if hai_i.is_some() && version.is_some() {
                if hai_end.is_none() {
                    hai_end = Some(token_i);
                }
                continue;
            } else {
                panic!("Unexpected newline token");
            }
        }

        if let ParsedToken::Word(token) = token {
            if hai_i.is_some() && version.is_none() {
                version = Some(token_i);
                continue;
            } else {
                panic!("Unexpected word {token}");
            }
        }

        if hai_i.is_some() && version.is_some() && hai_end.is_some() {
            break;
        }
    }

    if hai_i.is_none() && hai_end.is_none() {
        panic!("HAI missing")
    }

    if version.is_none() {
        panic!("Version missing")
    }

    let hai_end = hai_end.unwrap();
    for _ in 0..hai_end + 1 {
        tokens.remove(0);
    }

    let mut bye: Option<usize> = None;
    for (i, token) in tokens.iter().rev().enumerate() {
        match token {
            &ParsedToken::NewLine | &ParsedToken::Space => continue,
            ParsedToken::Keyword(KeywordToken::KThxBye) => {
                bye = Some(i);
                break;
            }
            _ => panic!("Unexpected token {:?}", token),
        }
    }

    if bye.is_none() {
        panic!("Missing KThxBye");
    }

    let bye = bye.unwrap();

    for _ in 0..bye + 1 {
        tokens.pop();
    }
    tokens
}

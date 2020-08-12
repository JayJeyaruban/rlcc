use std::collections::HashSet;

use crate::tokenizer::TokenParseResult::{TOKEN, VAL};

const HAI: &str = "HAI";
const KTHXBYE: &str = "KTHXBYE";
const SPACE: &str = " ";
const NL: &str = "\n";

lazy_static! {
    static ref TOKENS: HashSet<String> = vec![HAI, KTHXBYE, SPACE, NL]
        .into_iter()
        .map(|token| -> String { token.to_string() })
        .collect();
}

#[derive(Debug)]
pub enum TokenParseResult {
    TOKEN(String),
    VAL(String),
}

impl TokenParseResult {
    fn parse(s: &String) -> TokenParseResult {
        let s_str = s.as_str();
        if TOKENS.contains(s_str) {
            TOKEN(s.clone())
        } else {
            VAL(s.clone())
        }
    }
}

pub fn parse_tokens(content_string: &String) -> Vec<TokenParseResult> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    for c in content_string.chars() {
        match c {
            ' ' | '\n' => {
                if !current_word.is_empty() {
                    tokens.push(TokenParseResult::parse(&current_word));
                    current_word.clear();
                    tokens.push(TokenParseResult::parse(&c.to_string()));
                }
            }
            _ => current_word.push(c),
        }
    }

    tokens
}

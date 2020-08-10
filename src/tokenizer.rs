use crate::tokenizer::Token::VAL;

#[derive(Debug)]
pub enum Token {
    HAI,
    VAL(String),
    NL,
    KTHXBYE,
}

impl Token {
    fn parse(s: String) -> Token {
        VAL(s)
    }
}

pub fn parse_tokens(content_string: &String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    for c in content_string.chars() {
        match c {
            ' ' => {
                tokens.push(Token::parse(current_word));
                current_word = String::new();
            }
            _ => current_word.push(c)
        }
    }

    tokens
}

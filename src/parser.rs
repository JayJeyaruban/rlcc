use crate::tokenizer::{TokenParseResult, HAI};

pub fn parse_tokens(tokens: &Vec<TokenParseResult>) {
  println!("{:?}", tokens);

  for (i, token) in tokens.iter().enumerate() {
    if i == 0 {
      if let TokenParseResult::Token(t) = token {
        if t == HAI {
          continue;
        } else {
          panic!(format!("Unexpected token {}", t));
        }
      } else {
        panic!(format!("Unexpected token {:?}", token));
      }
    }
  }

  println!("Successfully able to parse file.");
}

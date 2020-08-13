use crate::tokenizer::{TokenParseResult, HAI};

const LATEST_LOLCODE_VERSION: f32 = 1.3;

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
    } else if i == 1 {
      if let TokenParseResult::Val(val) = token {
        if let Result::Ok(v) = val.parse::<f32>() {
          if v > 0.0 && v <= LATEST_LOLCODE_VERSION {
            continue;
          }
        }
      }
      panic!(format!("Unexpected token {:?}", token));
    }
  }

  println!("Successfully able to parse file.");
}

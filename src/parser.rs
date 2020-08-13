use crate::tokenizer::{TokenParseResult, HAI, KTHXBYE};

const LATEST_LOLCODE_VERSION: f32 = 1.3;

pub fn parse_tokens(tokens: &Vec<TokenParseResult>) {
  println!("{:?}", tokens);

  let filtered_tokens = tokens
    .into_iter()
    .filter(|token| {
      !if let TokenParseResult::Token(t) = token {
        t == " "
      } else {
        false
      }
    })
    .collect::<Vec<&TokenParseResult>>();

  for (i, token) in filtered_tokens.iter().enumerate() {
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
    } else if i as usize == filtered_tokens.len() - 1 {
      if let TokenParseResult::Token(t) = token {
        if t == KTHXBYE {
          continue;
        }
      }
      panic!(format!("Unexpected token {:?}", token));
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

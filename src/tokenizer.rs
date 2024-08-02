use derive_more::Display;
use mediator_tracing::tracing::trace;

#[derive(Debug, PartialEq, Eq, Display)]
pub enum KeywordToken {
    Hai,
    KThxBye,
    Visible,
    Btw,
    Can,
    Has,
}

impl From<KeywordToken> for ParsedToken {
    fn from(value: KeywordToken) -> Self {
        ParsedToken::Keyword(value)
    }
}

#[derive(Debug, PartialEq, Eq, Display)]
pub enum ParsedToken {
    Keyword(KeywordToken),
    #[display(fmt = "{_0}")]
    Word(String),
    #[display(fmt = " ")]
    Space,
    #[display(fmt = "\n")]
    NewLine,
    #[display(fmt = ".")]
    Period,
    #[display(fmt = ",")]
    Comma,
    #[display(fmt = "\"")]
    Quote,
}

#[derive(Debug)]
pub struct TokenLocation {
    line: usize,
    column: usize,
}

#[derive(Debug)]
#[jsm::public]
pub struct TokenParseResult {
    location: TokenLocation,
    result: ParsedToken,
}

pub fn parse_tokens(content_string: String) -> Vec<TokenParseResult> {
    let mut parsed_tokens = Vec::new();
    let mut buffer: String = String::new();
    let mut skip_nl = false;
    let mut current_line = 1;
    let mut current_col = 1;
    for c in content_string.chars() {
        let mut consume_buffer_and_append = |token| {
            let mut tokens = vec![];
            if let Some(token) = parse_word(&mut buffer) {
                tokens.push(token);
            }
            tokens.push(token);
            tokens
        };

        let tokens;
        (tokens, current_line, current_col) = match c {
            '\r' => {
                skip_nl = true;
                let tokens = consume_buffer_and_append(ParsedToken::NewLine);
                (Some(tokens), current_line + 1, 1)
            }
            '\n' => match skip_nl {
                true => (None, current_line, current_col),
                false => {
                    let tokens = consume_buffer_and_append(ParsedToken::NewLine);
                    (Some(tokens), current_line + 1, 1)
                }
            },
            ' ' | '\t' => {
                let tokens = consume_buffer_and_append(ParsedToken::Space);
                (Some(tokens), current_line, current_col + 1)
            }
            '.' => {
                let tokens = consume_buffer_and_append(ParsedToken::Period);
                (Some(tokens), current_line, current_col + 1)
            }
            ',' => {
                let tokens = consume_buffer_and_append(ParsedToken::Comma);
                (Some(tokens), current_line, current_col + 1)
            }
            '"' => {
                let tokens = consume_buffer_and_append(ParsedToken::Quote);
                (Some(tokens), current_line, current_col + 1)
            }
            _ => {
                buffer.push(c);
                (None, current_line, current_col + 1)
            }
        };
        if let Some(tokens) = tokens {
            for token in tokens {
                parsed_tokens.push(TokenParseResult {
                    location: TokenLocation {
                        line: current_line,
                        column: current_col,
                    },
                    result: token,
                });
            }
        }
    }

    parsed_tokens
}

fn parse_word(buffer: &mut String) -> Option<ParsedToken> {
    let token = match buffer.as_str() {
        "HAI" => Some(KeywordToken::Hai.into()),
        "KTHXBYE" => Some(KeywordToken::KThxBye.into()),
        "VISIBLE" => Some(KeywordToken::Visible.into()),
        "BTW" => Some(KeywordToken::Btw.into()),
        "CAN" => Some(KeywordToken::Can.into()),
        "HAS" => Some(KeywordToken::Has.into()),
        "" => None,
        _ => Some(ParsedToken::Word(buffer.to_string())),
    };
    buffer.clear();
    trace!(?token);
    token
}

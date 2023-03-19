#[derive(Debug, PartialEq, Eq)]
pub enum KeywordToken {
    Hai,
    KThxBye,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedToken {
    Keyword(KeywordToken),
    Word(String),
    Space,
    NewLine,
}

pub type TokenParseResult = Result<ParsedToken, String>;

pub fn parse_tokens(content_string: String) -> Vec<TokenParseResult> {
    let mut parsed_tokens = Vec::new();
    let mut buffer: Option<String> = None;
    for c in content_string.chars() {
        let tokens;
        (buffer, tokens) = match buffer {
            None => match c {
                ' ' => (None, vec![Ok(ParsedToken::Space)]),
                '\n' => (None, vec![Ok(ParsedToken::NewLine)]),
                _ => (Some(String::from(c)), vec![]),
            },
            Some(mut buffer) => match c {
                ' ' => (None, vec![parse_token(buffer), Ok(ParsedToken::Space)]),
                '\n' => (None, vec![parse_token(buffer), Ok(ParsedToken::NewLine)]),
                _ => {
                    buffer.push(c);
                    (Some(buffer), vec![])
                }
            },
        };

        tokens
            .into_iter()
            .for_each(|token| parsed_tokens.push(token));
    }
    parsed_tokens
}

fn parse_token(buffer: String) -> TokenParseResult {
    match buffer.as_str() {
        "HAI" => Ok(ParsedToken::Keyword(KeywordToken::Hai)),
        "KTHXBYE" => Ok(ParsedToken::Keyword(KeywordToken::KThxBye)),
        _ => Ok(ParsedToken::Word(buffer)),
    }
}

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
    Period,
}

pub type TokenParseResult = Result<ParsedToken, String>;

pub fn parse_tokens(content_string: String) -> Vec<TokenParseResult> {
    let mut parsed_tokens = Vec::new();
    let mut buffer: Option<String> = None;
    let mut skip_nl = false;
    for c in content_string.chars() {
        if !skip_nl && c == '\r' {
            skip_nl = true;
        }
        let tokens;
        (buffer, tokens) = process_char(c, buffer, skip_nl);

        if let Some(tokens) = tokens {
            tokens
                .into_iter()
                .for_each(|token| parsed_tokens.push(token));
        }
    }
    parsed_tokens
}

fn process_char(
    c: char,
    buffer: Option<String>,
    skip_nl: bool,
) -> (Option<String>, Option<Vec<TokenParseResult>>) {
    let parse_buffer_and_append = |buffer, token| {
        let mut tokens = parse_word(buffer)
            .map(|token| vec![token])
            .unwrap_or_default();

        tokens.push(Ok(token));
        tokens
    };

    match c {
        ' ' | '\t' => (
            None,
            Some(parse_buffer_and_append(buffer, ParsedToken::Space)),
        ),
        '\n' => match skip_nl {
            true => (buffer, None),
            false => (
                None,
                Some(parse_buffer_and_append(buffer, ParsedToken::NewLine)),
            ),
        },
        '\r' => (
            None,
            Some(parse_buffer_and_append(buffer, ParsedToken::NewLine)),
        ),
        '.' => (
            None,
            Some(parse_buffer_and_append(buffer, ParsedToken::Period)),
        ),
        _ => {
            let mut buffer = buffer.unwrap_or_default();
            buffer.push(c);
            (Some(buffer), None)
        }
    }
}

fn parse_word(buffer: Option<String>) -> Option<TokenParseResult> {
    buffer.map(|buffer| match buffer.as_str() {
        "HAI" => Ok(ParsedToken::Keyword(KeywordToken::Hai)),
        "KTHXBYE" => Ok(ParsedToken::Keyword(KeywordToken::KThxBye)),
        _ => Ok(ParsedToken::Word(buffer)),
    })
}

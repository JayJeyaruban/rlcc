use crate::tokenizer::{KeywordToken, ParsedToken};

enum ScopeContext {
    None,
    Decoration,
}

pub fn process_tokens(tokens: Vec<ParsedToken>) -> Vec<String> {
    let mut context = ScopeContext::None;
    let mut errs = Vec::new();
    for token in tokens {
        let err;
        (context, err) = match context {
            ScopeContext::None => match token {
                ParsedToken::Space | ParsedToken::NewLine => (context, None),
                ParsedToken::Keyword(KeywordToken::Hai) => (ScopeContext::Decoration, None),
                _ => (
                    context,
                    Some(format!(
                        "Unexpected token {token:?}. Expected {:?}",
                        KeywordToken::Hai
                    )),
                ),
            },
            ScopeContext::Decoration => match token {
                ParsedToken::Keyword(KeywordToken::KThxBye) => (ScopeContext::None, None),
                ParsedToken::Keyword(KeywordToken::Hai) => {
                    (context, Some(format!("Unexpected token {token:?}")))
                }
                ParsedToken::NewLine | ParsedToken::Space => (context, None),
                _ => (context, None),
            },
        };

        if let Some(err) = err {
            errs.push(err);
        }
    }

    errs
}

use crate::tokenizer::{KeywordToken, ParsedToken};

struct LolCodeVersion {
    major: i32,
    minor: i32,
}

impl TryFrom<String> for LolCodeVersion {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split('.').collect();
        let major = parts.get(0).ok_or("Version does not contain major part")?;
        let major = major
            .parse::<_>()
            .map_err(|_| "Unable to parse major part")?;
        let minor = parts.get(1).ok_or("Version does not contain minor part")?;
        let minor = minor
            .parse::<_>()
            .map_err(|_| "Unable to parse minor part")?;

        Ok(Self { major, minor })
    }
}

enum ScopeContext {
    None,
    Decoration,
    DecorationWithVersion(LolCodeVersion),
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
                ParsedToken::Keyword(token) => {
                    (context, Some(format!("Unexpected token {token:?}")))
                }
                ParsedToken::Word(word) => match LolCodeVersion::try_from(word) {
                    Ok(version) => (ScopeContext::DecorationWithVersion(version), None),
                    Err(err) => (context, Some(err)),
                },
                ParsedToken::NewLine | ParsedToken::Space => (context, None),
                _ => (context, None),
            },
            ScopeContext::DecorationWithVersion(_) => match token {
                ParsedToken::Keyword(KeywordToken::KThxBye) => (ScopeContext::None, None),
                ParsedToken::Keyword(token) => {
                    (context, Some(format!("Unexpected token {token:?}")))
                }
                ParsedToken::Space | ParsedToken::NewLine => (context, None),
                _ => (context, None),
            },
        };

        if let Some(err) = err {
            errs.push(err);
        }
    }

    errs
}

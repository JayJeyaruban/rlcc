use tracing::{debug, error};

use crate::tokenizer::{KeywordToken, ParsedToken};

#[derive(Debug)]
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

impl TryFrom<(i32, i32)> for LolCodeVersion {
    type Error = String;

    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        let version = LolCodeVersion {
            major: value.0,
            minor: value.1,
        };
        debug!(?version);
        Ok(version)
    }
}

#[derive(Debug)]
enum ScopeContext {
    None,
    Decoration(DecorationContext),
    Join(JoinContext),
}

#[derive(Debug)]
enum DecorationContext {
    Started,
    WithMajor(i32),
    WithMajorAndPeriod(i32),
    WithVersion(LolCodeVersion),
}

#[derive(Debug)]
enum JoinContext {
    Period1,
    Period2,
    Period3,
    NextLine,
}

pub fn process_tokens(tokens: Vec<ParsedToken>) -> Vec<String> {
    let mut context = ScopeContext::None;
    let mut errs = Vec::new();
    for token in tokens {
        let err;
        debug!(?context);
        (context, err) = match context {
            ScopeContext::None => match token {
                ParsedToken::Space | ParsedToken::NewLine => (context, None),
                ParsedToken::Keyword(KeywordToken::Hai) => {
                    (ScopeContext::Decoration(DecorationContext::Started), None)
                }
                _ => (
                    context,
                    Some(format!(
                        "Unexpected token {token:?}. Expected {:?}",
                        KeywordToken::Hai
                    )),
                ),
            },
            ScopeContext::Decoration(DecorationContext::Started) => match token {
                ParsedToken::Keyword(token) => {
                    (context, Some(format!("Unexpected token {token:?}")))
                }
                ParsedToken::Word(word) => (
                    ScopeContext::Decoration(DecorationContext::WithMajor(
                        word.parse::<i32>().unwrap(),
                    )),
                    None,
                ),
                ParsedToken::NewLine | ParsedToken::Space => (context, None),
                _ => (context, None),
            },
            ScopeContext::Decoration(DecorationContext::WithMajor(major)) => match token {
                ParsedToken::Period => (
                    ScopeContext::Decoration(DecorationContext::WithMajorAndPeriod(major)),
                    None,
                ),
                _ => (
                    context,
                    Some(format!("Unexpected token {token:?}. Expected period")),
                ),
            },
            ScopeContext::Decoration(DecorationContext::WithMajorAndPeriod(major)) => match token {
                ParsedToken::Word(word) => (
                    ScopeContext::Decoration(DecorationContext::WithVersion(
                        LolCodeVersion::try_from((major, word.parse::<i32>().unwrap())).unwrap(),
                    )),
                    None,
                ),
                _ => (
                    context,
                    Some(format!(
                        "Unexpected token {token:?}. Expected minor version"
                    )),
                ),
            },
            ScopeContext::Decoration(DecorationContext::WithVersion(_)) => match token {
                ParsedToken::Keyword(KeywordToken::KThxBye) => (ScopeContext::None, None),
                ParsedToken::Keyword(token) => {
                    (context, Some(format!("Unexpected token {token:?}")))
                }
                ParsedToken::Space | ParsedToken::NewLine => (context, None),
                ParsedToken::Period => (ScopeContext::Join(JoinContext::Period1), None),
                _ => (context, None),
            },
            ScopeContext::Join(JoinContext::Period1) => match token {
                ParsedToken::Period => (ScopeContext::Join(JoinContext::Period2), None),
                _ => (
                    context,
                    Some(format!("Unexpected token {token:?}. Expected '.'.")),
                ),
            },
            ScopeContext::Join(JoinContext::Period2) => match token {
                ParsedToken::Period => (ScopeContext::Join(JoinContext::Period3), None),
                _ => (
                    context,
                    Some(format!("Unexpected token {token:?}. Expected '.'.")),
                ),
            },
            ScopeContext::Join(JoinContext::Period3) => match token {
                ParsedToken::NewLine => (ScopeContext::Join(JoinContext::NextLine), None),
                _ => (
                    context,
                    Some(format!("Unexpected token {token:?}. Expected newline.")),
                ),
            },
            ScopeContext::Join(JoinContext::NextLine) => match token {
                ParsedToken::Space => (context, None),
                ParsedToken::Keyword(_) | ParsedToken::Word(_) => (
                    ScopeContext::Decoration(DecorationContext::WithVersion(LolCodeVersion {
                        major: 1,
                        minor: 3,
                    })),
                    None,
                ),
                _ => (context, Some(format!("Invalid continuation after join"))),
            },
        };

        if let Some(err) = err {
            error!(err);
            errs.push(err);
        }
    }

    match context {
        ScopeContext::None => {}
        _ => errs.push(format!("Missing final token {:?}", KeywordToken::KThxBye)),
    }

    errs
}

use crate::tokenizer::{KeywordToken, ParsedToken};

use super::{LolCodeProgram, LolCodeVersion, ParseScope, StackOp};

#[derive(Debug, PartialEq, Eq)]
pub enum ScopeContext {
    Decoration(DecorationContext),
    Main(MainContext),
    Comment(CommentContext),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecorationContext {
    Started,
    WithMajor(i32),
    WithMajorAndPeriod(i32),
}

impl ParseScope for DecorationContext {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match self {
            DecorationContext::Started => match token {
                ParsedToken::Keyword(token) => (
                    StackOp::Retain(None),
                    Some(format!("Unexpected token {token:?}")),
                ),
                ParsedToken::Word(word) => (
                    StackOp::Replace(
                        DecorationContext::WithMajor(
                            word.parse::<i32>()
                                .expect(format!("parsing major from {}", word).as_str()),
                        )
                        .into(),
                    ),
                    None,
                ),
                ParsedToken::NewLine | ParsedToken::Space => (StackOp::Retain(None), None),
                _ => (StackOp::Retain(None), None),
            },
            DecorationContext::WithMajor(major) => match token {
                ParsedToken::Period => (
                    StackOp::Replace(DecorationContext::WithMajorAndPeriod(*major).into()),
                    None,
                ),
                _ => (
                    StackOp::Retain(None),
                    Some(format!("Unexpected token {token:?}. Expected period")),
                ),
            },
            DecorationContext::WithMajorAndPeriod(major) => match token {
                ParsedToken::Word(word) => (
                    StackOp::Replace(
                        MainContext::Root {
                            version: LolCodeVersion::try_from((
                                *major,
                                word.parse::<i32>().unwrap(),
                            ))
                            .expect("parse minor for LolCode version"),
                            instrs: Vec::new(),
                        }
                        .into(),
                    ),
                    None,
                ),
                _ => (
                    StackOp::Retain(None),
                    Some(format!(
                        "Unexpected token {token:?}. Expected minor version"
                    )),
                ),
            },
        }
    }
}

impl From<DecorationContext> for ScopeContext {
    fn from(value: DecorationContext) -> Self {
        ScopeContext::Decoration(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MainContext {
    Pre,
    Root {
        version: LolCodeVersion,
        instrs: Vec<ExprContext>,
    },
    Expr(ExprContext),
    Complete(LolCodeProgram),
}

impl ParseScope for MainContext {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match (self, token) {
            (MainContext::Pre, ParsedToken::Keyword(KeywordToken::Hai)) => {
                (StackOp::Replace(DecorationContext::Started.into()), None)
            }
            (MainContext::Pre, ParsedToken::Space | ParsedToken::NewLine) => {
                (StackOp::Retain(None), None)
            }
            (MainContext::Pre, token) => (
                StackOp::Retain(None),
                Some(format!(
                    "Unexpected token {token:?}. Expected {:?}",
                    KeywordToken::Hai
                )),
            ),
            (MainContext::Root { .. }, ParsedToken::Space | ParsedToken::NewLine) => {
                (StackOp::Retain(None), None)
            }
            (MainContext::Root { version, instrs }, ParsedToken::Keyword(token)) => {
                Self::root_handle_keyword(version, instrs, token)
            }
            (MainContext::Root { .. }, token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
            (MainContext::Expr(expr), token) => expr.process_token(token),
            (MainContext::Complete(_), ParsedToken::NewLine | ParsedToken::Space) => {
                (StackOp::Retain(None), None)
            }
            (MainContext::Complete(_), token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
        }
    }
}

impl MainContext {
    fn root_handle_keyword(
        version: &mut LolCodeVersion,
        instrs: &mut Vec<ExprContext>,
        token: KeywordToken,
    ) -> (StackOp, Option<String>) {
        match token {
            KeywordToken::KThxBye => (
                StackOp::Replace(
                    MainContext::Complete(LolCodeProgram {
                        version: version.to_owned(),
                        instrs: instrs.to_owned(),
                    })
                    .into(),
                ),
                None,
            ),
            KeywordToken::Visible => (
                StackOp::Retain(Some(ExprContext::Visible { args: Vec::new() }.into())),
                None,
            ),
            KeywordToken::Can => (
                StackOp::Retain(Some(IncludeInProgress::Started.into())),
                None,
            ),
            KeywordToken::Has => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}. Are you missing CAN?")),
            ),
            token => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
        }
    }
}

impl From<MainContext> for ScopeContext {
    fn from(value: MainContext) -> Self {
        ScopeContext::Main(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprContext {
    Join(JoinContext),
    Visible { args: Vec<String> },
    String(StringExprContext),
    IncludeInProgress(IncludeInProgress),
    Include(IncludesContext),
}

impl From<ExprContext> for ScopeContext {
    fn from(value: ExprContext) -> Self {
        MainContext::Expr(value).into()
    }
}

impl ParseScope for ExprContext {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match (self, token) {
            (ExprContext::Visible { .. }, ParsedToken::NewLine | ParsedToken::Comma) => {
                (StackOp::Unwind, None)
            }
            (ExprContext::Visible { .. }, ParsedToken::Quote) => (
                StackOp::Retain(Some(StringExprContext(String::new()).into())),
                None,
            ),
            (ExprContext::Visible { .. }, ParsedToken::Space) => (StackOp::Retain(None), None),
            (ExprContext::Visible { .. }, ParsedToken::Period) => {
                (StackOp::Retain(Some(JoinContext::Period1.into())), None)
            }
            (ExprContext::Visible { .. }, token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}.")),
            ),
            (ExprContext::String(string_ctx), token) => string_ctx.process_token(token),
            (ExprContext::Join(join_ctx), token) => join_ctx.process_token(token),
            (ExprContext::IncludeInProgress(includes), token) => includes.process_token(token),
            (ExprContext::Include(_), ParsedToken::NewLine) => (StackOp::Unwind, None),
            (ExprContext::Include(_), ParsedToken::Space) => (StackOp::Retain(None), None),
            (ExprContext::Include(_), token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct StringExprContext(pub String);

impl ParseScope for StringExprContext {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match token {
            ParsedToken::NewLine => (StackOp::Unwind, Some("Unexpected newline".to_string())),
            ParsedToken::Quote => (StackOp::Unwind, None),
            token => {
                self.0.push_str(token.to_string().as_str());
                (StackOp::Retain(None), None)
            }
        }
    }
}

impl From<StringExprContext> for ScopeContext {
    fn from(value: StringExprContext) -> Self {
        ExprContext::String(value).into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JoinContext {
    Period1,
    Period2,
    Period3,
    NewLine,
}

impl JoinContext {
    pub fn process_token(&self, token: ParsedToken) -> (StackOp, Option<String>) {
        match self {
            JoinContext::Period1 => match token {
                ParsedToken::Period => (StackOp::Replace(JoinContext::Period2.into()), None),
                _ => (
                    StackOp::Retain(None),
                    Some(format!("Unexpected token {token:?}. Expected '.'.")),
                ),
            },
            JoinContext::Period2 => match token {
                ParsedToken::Period => (StackOp::Replace(JoinContext::Period3.into()), None),
                _ => (
                    StackOp::Retain(None),
                    Some(format!("Unexpected token {token:?}. Expected '.'.")),
                ),
            },
            JoinContext::Period3 => match token {
                ParsedToken::NewLine => (StackOp::Replace(JoinContext::NewLine.into()), None),
                _ => (
                    StackOp::Retain(None),
                    Some(format!("Unexpected token {token:?}. Expected newline.")),
                ),
            },
            JoinContext::NewLine => match token {
                ParsedToken::NewLine => (
                    StackOp::Unwind,
                    Some("invalid newline after join".to_string()),
                ),
                _ => (StackOp::Unwind, None),
            },
        }
    }
}

impl From<JoinContext> for ScopeContext {
    fn from(value: JoinContext) -> Self {
        ExprContext::Join(value).into()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommentContext {
    Started,
    InProgress(Vec<String>),
}

impl From<CommentContext> for ScopeContext {
    fn from(value: CommentContext) -> Self {
        ScopeContext::Comment(value)
    }
}

impl ParseScope for CommentContext {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match (self, token) {
            (CommentContext::Started, token) => (
                StackOp::Replace(CommentContext::InProgress(vec![token.to_string()]).into()),
                None,
            ),
            (CommentContext::InProgress(_), ParsedToken::NewLine) => (StackOp::Unwind, None),
            (CommentContext::InProgress(txt), token) => {
                txt.push(token.to_string());
                (StackOp::Retain(None), None)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IncludesContext {
    module: String,
}

impl From<IncludesContext> for ScopeContext {
    fn from(value: IncludesContext) -> Self {
        ExprContext::Include(value).into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncludeInProgress {
    Started,
    ReadyHas,
    Has,
    ReadyModule,
}

impl From<IncludeInProgress> for ScopeContext {
    fn from(value: IncludeInProgress) -> Self {
        ExprContext::IncludeInProgress(value).into()
    }
}

impl ParseScope for IncludeInProgress {
    fn process_token(&mut self, token: ParsedToken) -> (StackOp, Option<String>) {
        match (self, token) {
            (IncludeInProgress::Started, ParsedToken::Space) => {
                (StackOp::Replace(IncludeInProgress::ReadyHas.into()), None)
            }
            (IncludeInProgress::Started, token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
            (IncludeInProgress::ReadyHas, ParsedToken::Keyword(KeywordToken::Has)) => {
                (StackOp::Replace(IncludeInProgress::Has.into()), None)
            }
            (IncludeInProgress::Has, ParsedToken::Space) => (
                StackOp::Replace(IncludeInProgress::ReadyModule.into()),
                None,
            ),
            (IncludeInProgress::ReadyModule, ParsedToken::Word(module)) => {
                (StackOp::Replace(IncludesContext { module }.into()), None)
            }
            (IncludeInProgress::ReadyModule, _) => (
                StackOp::Retain(None),
                Some("Expected module to include".to_string()),
            ),
            (_, token) => (
                StackOp::Retain(None),
                Some(format!("Unexpected token {token:?}")),
            ),
        }
    }
}

use crate::{
    framework::{HandleTokenProcessingError, TokenProcessingError},
    tokenizer::{KeywordToken, Token, TokenType},
};

use super::{Instruction, LolCodeProgram, LolCodeVersion, ParseScope, StackOp};

#[derive(Debug, PartialEq, Eq)]
pub enum ScopeContext {
    Decoration(DecorationContext),
    Main(MainContext),
    Comment(CommentContext),
}

impl<T> ParseScope<ScopeContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(
        &mut self,
        scope: &mut ScopeContext,
        token: &Token,
    ) -> anyhow::Result<StackOp> {
        match scope {
            ScopeContext::Comment(comment) => self.process_token(comment, token),
            ScopeContext::Decoration(decoration) => self.process_token(decoration, token),
            ScopeContext::Main(main) => self.process_token(main, token),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecorationContext {
    Started,
    WithMajor(i32),
    WithMajorAndPeriod(i32),
}

impl<T> ParseScope<DecorationContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(
        &mut self,
        scope: &mut DecorationContext,
        token: &Token,
    ) -> anyhow::Result<StackOp> {
        let op = match scope {
            DecorationContext::Started => match &token.t_type {
                TokenType::Keyword(keyword) => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {keyword:?}"),
                    })?;
                    StackOp::Retain(None)
                }
                TokenType::Word(word) => StackOp::Replace(
                    DecorationContext::WithMajor(
                        word.parse::<i32>()
                            .expect(format!("parsing major from {}", word).as_str()),
                    )
                    .into(),
                ),
                TokenType::NewLine | TokenType::Space => StackOp::Retain(None),
                _ => StackOp::Retain(None),
            },
            DecorationContext::WithMajor(major) => match token.t_type {
                TokenType::Period => {
                    StackOp::Replace(DecorationContext::WithMajorAndPeriod(*major).into())
                }
                _ => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {token:?}. Expected period"),
                    })?;
                    StackOp::Retain(None)
                }
            },
            DecorationContext::WithMajorAndPeriod(major) => match &token.t_type {
                TokenType::Word(word) => StackOp::Replace(
                    MainContext::Root {
                        version: LolCodeVersion::try_from((*major, word.parse::<i32>().unwrap()))
                            .expect("parse minor for LolCode version"),
                        instrs: Vec::new(),
                    }
                    .into(),
                ),
                _ => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {token:?}. Expected minor version"),
                    })?;
                    StackOp::Retain(None)
                }
            },
        };
        Ok(op)
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
        instrs: Vec<Instruction>,
    },
    Expr(ExprContext),
    Complete(LolCodeProgram),
}

impl<T> ParseScope<MainContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(&mut self, scope: &mut MainContext, token: &Token) -> anyhow::Result<StackOp> {
        let op = match (scope, &token.t_type) {
            (MainContext::Pre, TokenType::Keyword(KeywordToken::Hai)) => {
                StackOp::Replace(DecorationContext::Started.into())
            }
            (MainContext::Pre, TokenType::Space | TokenType::NewLine) => StackOp::Retain(None),
            (MainContext::Pre, t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!(
                        "Unexpected token {t_type:?}. Expected {:?}",
                        KeywordToken::Hai
                    ),
                })?;
                StackOp::Retain(None)
            }
            (MainContext::Root { .. }, TokenType::Space | TokenType::NewLine) => {
                StackOp::Retain(None)
            }
            (MainContext::Root { version, instrs }, TokenType::Keyword(kw_token)) => {
                MainContext::root_handle_keyword(version, instrs, kw_token, |err| {
                    self.handle_err(TokenProcessingError { token, err })
                })?
            }
            (MainContext::Root { .. }, t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}"),
                })?;
                StackOp::Retain(None)
            }
            (MainContext::Expr(expr), _) => self.process_token(expr, token)?,
            (MainContext::Complete(_), TokenType::NewLine | TokenType::Space) => {
                StackOp::Retain(None)
            }
            (MainContext::Complete(_), t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}"),
                })?;
                StackOp::Retain(None)
            }
        };
        Ok(op)
    }
}

impl MainContext {
    fn root_handle_keyword<F>(
        version: &mut LolCodeVersion,
        instrs: &mut Vec<Instruction>,
        token: &KeywordToken,
        mut handle_err: F,
    ) -> anyhow::Result<StackOp>
    where
        F: FnMut(String) -> anyhow::Result<()>,
    {
        let op = match token {
            KeywordToken::KThxBye => StackOp::Replace(
                MainContext::Complete(LolCodeProgram {
                    version: version.to_owned(),
                    instrs: instrs.to_owned(),
                })
                .into(),
            ),
            KeywordToken::Visible => {
                StackOp::Retain(Some(ExprContext::Visible { args: Vec::new() }.into()))
            }
            KeywordToken::Can => StackOp::Retain(Some(IncludesContext::Started.into())),
            KeywordToken::Has => {
                handle_err(format!("Unexpected token {token:?}. Are you missing CAN?"))?;
                StackOp::Retain(None)
            }
            token => {
                handle_err(format!("Unexpected token {token:?}"))?;
                StackOp::Retain(None)
            }
        };
        Ok(op)
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
    Include(IncludesContext),
}

impl From<ExprContext> for ScopeContext {
    fn from(value: ExprContext) -> Self {
        MainContext::Expr(value).into()
    }
}

impl<T> ParseScope<ExprContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(&mut self, scope: &mut ExprContext, token: &Token) -> anyhow::Result<StackOp> {
        let op = match (scope, &token.t_type) {
            (ExprContext::Visible { .. }, TokenType::NewLine | TokenType::Comma) => StackOp::Unwind,
            (ExprContext::Visible { .. }, TokenType::Quote) => {
                StackOp::Retain(Some(StringExprContext(String::new()).into()))
            }
            (ExprContext::Visible { .. }, TokenType::Space) => StackOp::Retain(None),
            (ExprContext::Visible { .. }, TokenType::Period) => {
                StackOp::Retain(Some(JoinContext::Period1.into()))
            }
            (ExprContext::Visible { .. }, t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}."),
                })?;
                StackOp::Retain(None)
            }
            (ExprContext::String(string_ctx), _) => self.process_token(string_ctx, token)?,
            (ExprContext::Join(join_ctx), _) => self.process_token(join_ctx, token)?,
            (ExprContext::Include(includes), _) => self.process_token(includes, token)?,
            (ExprContext::Include(_), TokenType::NewLine) => StackOp::Unwind,
            (ExprContext::Include(_), TokenType::Space) => StackOp::Retain(None),
            (ExprContext::Include(_), t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}"),
                })?;
                StackOp::Retain(None)
            }
        };
        Ok(op)
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct StringExprContext(pub String);

impl<T> ParseScope<StringExprContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(
        &mut self,
        scope: &mut StringExprContext,
        token: &Token,
    ) -> anyhow::Result<StackOp> {
        let op = match &token.t_type {
            TokenType::NewLine => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: "Unexpected newline".to_string(),
                })?;
                StackOp::Unwind
            }
            TokenType::Quote => StackOp::Unwind,
            token => {
                scope.0.push_str(token.to_string().as_str());
                StackOp::Retain(None)
            }
        };
        Ok(op)
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

impl<T> ParseScope<JoinContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(&mut self, scope: &mut JoinContext, token: &Token) -> anyhow::Result<StackOp> {
        let op = match scope {
            JoinContext::Period1 => match token.t_type {
                TokenType::Period => StackOp::Replace(JoinContext::Period2.into()),
                _ => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {token:?}. Expected '.'."),
                    })?;
                    StackOp::Retain(None)
                }
            },
            JoinContext::Period2 => match &token.t_type {
                TokenType::Period => StackOp::Replace(JoinContext::Period3.into()),
                t_type => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {t_type:?}. Expected '.'."),
                    })?;
                    StackOp::Retain(None)
                }
            },
            JoinContext::Period3 => match &token.t_type {
                TokenType::NewLine => StackOp::Replace(JoinContext::NewLine.into()),
                t_type => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: format!("Unexpected token {t_type:?}. Expected newline."),
                    })?;
                    StackOp::Retain(None)
                }
            },
            JoinContext::NewLine => match token.t_type {
                TokenType::NewLine => {
                    self.handle_err(TokenProcessingError {
                        token,
                        err: "invalid newline after join".to_string(),
                    })?;
                    StackOp::Unwind
                }
                _ => StackOp::Unwind,
            },
        };
        Ok(op)
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

impl<T> ParseScope<CommentContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(
        &mut self,
        scope: &mut CommentContext,
        token: &Token,
    ) -> anyhow::Result<StackOp> {
        let op = match (scope, &token.t_type) {
            (CommentContext::Started, t_type) => {
                StackOp::Replace(CommentContext::InProgress(vec![t_type.to_string()]).into())
            }
            (CommentContext::InProgress(_), TokenType::NewLine) => StackOp::Unwind,
            (CommentContext::InProgress(txt), token) => {
                txt.push(token.to_string());
                StackOp::Retain(None)
            }
        };
        Ok(op)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IncludesContext {
    Started,
    ReadyHas,
    Has,
    ReadyModule,
    Module(String),
}

impl From<IncludesContext> for ScopeContext {
    fn from(value: IncludesContext) -> Self {
        ExprContext::Include(value).into()
    }
}

impl<T> ParseScope<IncludesContext> for T
where
    T: HandleTokenProcessingError,
{
    fn process_token(
        &mut self,
        scope: &mut IncludesContext,
        token: &Token,
    ) -> anyhow::Result<StackOp> {
        let op = match (scope, &token.t_type) {
            (IncludesContext::Started, TokenType::Space) => {
                StackOp::Replace(IncludesContext::ReadyHas.into())
            }
            (IncludesContext::Started, t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}"),
                })?;
                StackOp::Retain(None)
            }
            (IncludesContext::ReadyHas, TokenType::Keyword(KeywordToken::Has)) => {
                StackOp::Replace(IncludesContext::Has.into())
            }
            (IncludesContext::Has, TokenType::Space) => {
                StackOp::Replace(IncludesContext::ReadyModule.into())
            }
            (IncludesContext::ReadyModule, TokenType::Word(module)) => {
                StackOp::Replace(IncludesContext::Module(module.to_owned()).into())
            }
            (IncludesContext::Module(_), TokenType::NewLine) => StackOp::Unwind,
            (IncludesContext::ReadyModule, _) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: "Expected module to include".to_string(),
                })?;
                StackOp::Retain(None)
            }
            (_, t_type) => {
                self.handle_err(TokenProcessingError {
                    token,
                    err: format!("Unexpected token {t_type:?}"),
                })?;
                StackOp::Retain(None)
            }
        };
        Ok(op)
    }
}

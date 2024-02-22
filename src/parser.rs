use mediator_tracing::tracing::{debug, error};

use crate::tokenizer::{KeywordToken, ParsedToken};

#[derive(Debug, PartialEq, Eq, Clone)]
#[jsm::public]
pub struct LolCodeVersion {
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

impl From<(i32, i32)> for LolCodeVersion {
    fn from(value: (i32, i32)) -> Self {
        let version = LolCodeVersion {
            major: value.0,
            minor: value.1,
        };
        debug!(?version);
        version
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ScopeContext {
    Decoration(DecorationContext),
    Main(MainContext),
}

#[derive(Debug)]
enum StackOp {
    Unwind,
    Retain(Option<ScopeContext>),
    Replace(ScopeContext),
}

#[derive(Debug, PartialEq, Eq)]
enum DecorationContext {
    Started,
    WithMajor(i32),
    WithMajorAndPeriod(i32),
}

impl DecorationContext {
    fn process_token(&self, token: ParsedToken) -> (StackOp, Option<String>) {
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
enum MainContext {
    Root {
        version: LolCodeVersion,
        instrs: Vec<ExprContext>,
    },
    Expr(ExprContext),
    Complete(LolCodeProgram),
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
}

impl From<ExprContext> for ScopeContext {
    fn from(value: ExprContext) -> Self {
        MainContext::Expr(value).into()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct StringExprContext(pub String);

impl StringExprContext {
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
    fn process_token(&self, token: ParsedToken) -> (StackOp, Option<String>) {
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
#[jsm::public]
pub struct LolCodeProgram {
    version: LolCodeVersion,
    instrs: Vec<ExprContext>,
}

pub fn process_tokens(tokens: Vec<ParsedToken>) -> (Option<LolCodeProgram>, Vec<String>) {
    let mut ctx_stack = vec![];

    let mut errs = Vec::new();
    for token in tokens {
        let mut context = ctx_stack.pop();
        debug!(?context, ?token);
        let (op, err) = match context {
            None => match token {
                ParsedToken::Space | ParsedToken::NewLine => (StackOp::Retain(None), None),
                ParsedToken::Keyword(KeywordToken::Hai) => {
                    (StackOp::Replace(DecorationContext::Started.into()), None)
                }
                _ => (
                    StackOp::Retain(None),
                    Some(format!(
                        "Unexpected token {token:?}. Expected {:?}",
                        KeywordToken::Hai
                    )),
                ),
            },
            Some(ref mut context) => match context {
                ScopeContext::Decoration(ref decoration) => decoration.process_token(token),
                ScopeContext::Main(MainContext::Expr(ExprContext::Visible { .. })) => match token {
                    ParsedToken::NewLine | ParsedToken::Comma => (StackOp::Unwind, None),
                    ParsedToken::Quote => (
                        StackOp::Retain(Some(StringExprContext(String::new()).into())),
                        None,
                    ),
                    ParsedToken::Space => (StackOp::Retain(None), None),
                    ParsedToken::Period => {
                        (StackOp::Retain(Some(JoinContext::Period1.into())), None)
                    }
                    _ => (
                        StackOp::Retain(None),
                        Some(format!("Unexpected token {token:?}.")),
                    ),
                },
                ScopeContext::Main(MainContext::Expr(ExprContext::String(ref mut string_ctx))) => {
                    string_ctx.process_token(token)
                }
                ScopeContext::Main(MainContext::Expr(ExprContext::Join(ref join))) => {
                    join.process_token(token)
                }
                ScopeContext::Main(MainContext::Root {
                    ref version,
                    ref instrs,
                }) => match token {
                    ParsedToken::Keyword(KeywordToken::KThxBye) => (
                        StackOp::Replace(
                            MainContext::Complete(LolCodeProgram {
                                version: version.to_owned(),
                                instrs: instrs.to_owned(),
                            })
                            .into(),
                        ),
                        None,
                    ),
                    ParsedToken::Keyword(KeywordToken::Visible) => (
                        StackOp::Retain(Some(ExprContext::Visible { args: Vec::new() }.into())),
                        None,
                    ),
                    ParsedToken::Keyword(token) => (
                        StackOp::Retain(None),
                        Some(format!("Unexpected token {token:?}")),
                    ),
                    ParsedToken::Space | ParsedToken::NewLine => (StackOp::Retain(None), None),
                    _ => (StackOp::Retain(None), None),
                },
                ScopeContext::Main(MainContext::Complete(_)) => match token {
                    ParsedToken::Space | ParsedToken::NewLine => (StackOp::Retain(None), None),
                    token => (
                        StackOp::Retain(None),
                        Some(format!("Unexpected token {token:?}")),
                    ),
                },
            },
        };

        match op {
            StackOp::Unwind => {
                let mut next = ctx_stack.pop().expect("popping main ctx");
                let context = context.expect("expecting current context");
                debug!(?context, ?next, "performing unwind");
                match (&mut next, context) {
                    (_, ScopeContext::Main(MainContext::Root { .. })) => {}
                    (
                        ScopeContext::Main(MainContext::Root {
                            instrs: ref mut exprs,
                            ..
                        }),
                        ScopeContext::Main(MainContext::Expr(expr)),
                    ) => {
                        exprs.push(expr);
                    }
                    (
                        ScopeContext::Main(MainContext::Expr(ExprContext::Visible { args })),
                        ScopeContext::Main(MainContext::Expr(ExprContext::String(string))),
                    ) => args.push(string.0),
                    (
                        ScopeContext::Main(MainContext::Expr(ExprContext::Visible { .. })),
                        ScopeContext::Main(MainContext::Expr(ExprContext::Join(
                            JoinContext::NewLine,
                        ))),
                    ) => {}
                    _ => panic!("unexpected current and previous ctx"),
                }

                ctx_stack.push(next);
            }
            StackOp::Retain(next) => {
                if let Some(context) = context {
                    ctx_stack.push(context);
                }
                if let Some(next) = next {
                    ctx_stack.push(next);
                }
            }
            StackOp::Replace(next) => {
                ctx_stack.push(next);
            }
        }

        if let Some(err) = err {
            error!(err);
            errs.push(err);
        }
    }

    debug!(?ctx_stack);

    if ctx_stack.len() != 1 {
        errs.push(format!("Stack is inappropriate length"));
    }

    match ctx_stack.pop() {
        Some(ScopeContext::Main(MainContext::Complete(program))) => (Some(program), errs),
        Some(ctx) => {
            errs.push(format!("Unexpected ctx popped from stack {ctx:?}"));
            (None, errs)
        }
        None => (None, errs),
    }
}

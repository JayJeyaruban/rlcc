mod scope;

use anyhow::{bail, Context};
use mediator_tracing::tracing::{debug, debug_span};
use scope::*;

pub use scope::ExprContext;

use crate::{
    framework::HandleTokenProcessingError,
    tokenizer::{KeywordToken, Token, TokenType},
};

pub trait Parser {
    fn process_tokens(&mut self, tokens: Vec<Token>) -> anyhow::Result<LolCodeProgram>;
}

impl<T> Parser for T
where
    T: HandleTokenProcessingError,
{
    fn process_tokens(&mut self, tokens: Vec<Token>) -> anyhow::Result<LolCodeProgram> {
        let mut ctx_stack: Vec<ScopeContext> = vec![];
        ctx_stack.push(MainContext::Pre.into());

        for token in tokens {
            let _ = debug_span!("Process token", ?ctx_stack).entered();
            let mut context = ctx_stack.pop().context("non-empty ctx stack")?;
            debug!(?context, token = ?token.t_type);

            let op = match &token.t_type {
                TokenType::Keyword(KeywordToken::Btw) => {
                    StackOp::Retain(Some(CommentContext::Started.into()))
                }
                _ => self.process_token(&mut context, &token)?,
            };

            execute_stack_op(op, &mut ctx_stack, context);
        }

        debug!(?ctx_stack);

        if ctx_stack.len() != 1 {
            bail!("Stack is inappropriate length")
        }

        match ctx_stack.pop() {
            Some(ScopeContext::Main(MainContext::Complete(program))) => Ok(program),
            Some(ctx) => bail!("Unexpected ctx popped from stack {ctx:?}"),
            None => unreachable!("stack should have length 1"),
        }
    }
}

fn execute_stack_op(
    op: StackOp,
    ctx_stack: &mut Vec<ScopeContext>,
    context: ScopeContext,
) -> Option<ScopeContext> {
    match op {
        StackOp::Unwind => {
            let next = ctx_stack.last_mut().expect("popping main ctx");
            debug!(?context, ?next, "performing unwind");
            match (next, context) {
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
                    ScopeContext::Main(MainContext::Expr(ExprContext::Visible { ref mut args })),
                    ScopeContext::Main(MainContext::Expr(ExprContext::String(string))),
                ) => args.push(string.0),
                (
                    ScopeContext::Main(MainContext::Expr(ExprContext::Visible { .. })),
                    ScopeContext::Main(MainContext::Expr(ExprContext::Join(JoinContext::NewLine))),
                ) => {}
                (next, ScopeContext::Comment(CommentContext::InProgress(txt))) => {
                    debug!(?txt, "Dropping comment text");
                    match next {
                        ScopeContext::Main(MainContext::Expr(_)) => {
                            let n = ctx_stack.pop().expect("next exists from peek");
                            let n = execute_stack_op(op, ctx_stack, n);
                            if let Some(n) = n {
                                ctx_stack.push(n);
                            }
                        }
                        _ => {}
                    }
                }
                _ => panic!("unexpected current and previous ctx"),
            }
        }
        StackOp::Retain(next) => {
            ctx_stack.push(context);
            if let Some(next) = next {
                ctx_stack.push(next);
            }
        }
        StackOp::Replace(next) => {
            ctx_stack.push(next);
        }
    }

    None
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[jsm::public]
pub struct LolCodeVersion {
    major: i32,
    minor: i32,
}

#[derive(Debug, PartialEq, Eq)]
#[jsm::public]
pub struct LolCodeProgram {
    version: LolCodeVersion,
    instrs: Vec<ExprContext>,
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

#[derive(Debug)]
pub enum StackOp {
    Unwind,
    Retain(Option<ScopeContext>),
    Replace(ScopeContext),
}

trait ParseScope<Scope> {
    fn process_token(&mut self, scope: &mut Scope, token: &Token) -> anyhow::Result<StackOp>;
}

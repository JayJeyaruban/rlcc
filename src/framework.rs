use std::io::{BufWriter, Write};

use anyhow::Context;

use crate::tokenizer::Token;

pub struct App<Out, Err> {
    out: Out,
    err: Err,
    error_handled: bool,
}

impl<O, E> App<BufWriter<O>, BufWriter<E>>
where
    O: Write,
    E: Write,
{
    pub fn new(out: O, err: E) -> Self {
        Self {
            out: BufWriter::new(out),
            err: BufWriter::new(err),
            error_handled: false,
        }
    }
}

pub trait StdOut {
    type Out: Write;
    fn out(&mut self) -> &mut Self::Out;
}

impl<O, Err> StdOut for App<O, Err>
where
    O: Write,
{
    type Out = O;

    fn out(&mut self) -> &mut Self::Out {
        &mut self.out
    }
}

#[jsm::public]
pub struct TokenProcessingError<'a> {
    token: &'a Token,
    err: String,
}

pub trait HandleTokenProcessingError {
    fn handle_err(&mut self, err: TokenProcessingError) -> anyhow::Result<()>;

    fn error_handled(&self) -> bool;
}

impl<O, E> HandleTokenProcessingError for App<O, E>
where
    E: Write,
{
    fn handle_err(&mut self, err: TokenProcessingError) -> anyhow::Result<()> {
        self.error_handled = true;
        let loc = &err.token.location;
        write!(self.err, "@{}:{} -> {}\n", loc.line, loc.column, err.err)
            .context("writing token error")
    }

    fn error_handled(&self) -> bool {
        self.error_handled
    }
}

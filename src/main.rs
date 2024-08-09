mod framework;
mod interpreter;
mod parser;
mod tokenizer;

#[cfg(test)]
mod test;

use anyhow::bail;
use clap::Parser as _;
use framework::{App, HandleTokenProcessingError};
use mediator::Module;
use mediator_tracing::tracing::{info, Level};
use mediator_tracing::{tracing::debug, TracingConfig};
use mediator_tracing::{Targets, TracingModule};
use parser::Parser;
use std::io::{stderr, stdout, Write};
use std::str::FromStr;
use std::{fs, path::Path, process::ExitCode};

use crate::interpreter::Interpret;
use crate::tokenizer::parse_tokens;

impl<StdOut, StdErr> App<StdOut, StdErr>
where
    StdOut: Write,
    StdErr: Write,
{
    fn run<P>(&mut self, path: P, mode: Mode) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let file_contents = fs::read_to_string(path).expect("Unable to read provided file");
        let tokens = parse_tokens(file_contents);

        debug!(tokens = ?(tokens.iter().map(|token| &token.t_type).collect::<Vec<_>>()));

        let prog = self.process_tokens(tokens)?;

        debug!(?prog);

        if self.error_handled() {
            bail!("Something went wrong when compiling.");
        }

        if mode == Mode::Interpret {
            self.execute(prog)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();
    TracingModule::new(Some(TracingConfig {
        base_targets: Some(
            Targets::default().with_default(Level::from_str(&args.log_level).unwrap()),
        ),
        layer: mediator_tracing::TracingLayer::Log,
    }))
    .initialize(None)
    .init();
    info!(?args);

    App::new(stdout(), stderr())
        .run(args.filename, args.mode)
        .map(|_| {
            println!("Compilation successful");
            ExitCode::SUCCESS
        })
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filename: String,
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
    /// Mode to execute
    #[arg(value_enum, default_value_t = Mode::Interpret)]
    mode: Mode,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum Mode {
    /// Use the interpreter
    Interpret,
}

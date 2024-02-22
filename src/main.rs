mod interpreter;
mod parser;
mod tokenizer;

use clap::Parser;
use mediator::Module;
use mediator_tracing::tracing::{info, Level};
use mediator_tracing::{tracing::debug, TracingConfig};
use mediator_tracing::{Targets, TracingModule};
use parser::process_tokens;
use std::io::{stdout, Write};
use std::str::FromStr;
use std::{fs, path::Path, process::ExitCode};
use tokenizer::{ParsedToken, TokenParseResult};

use crate::interpreter::execute;
use crate::tokenizer::parse_tokens;

fn run<P: AsRef<Path>, W: Write>(path: P, interpret: bool, mut out: W) -> Result<(), Vec<String>> {
    let file_contents = fs::read_to_string(path).expect("Unable to read provided file");
    let parse_results = parse_tokens(file_contents);

    let (tokens, errs) = split_errs(parse_results);
    debug!(tokens = ?&tokens);

    let mut errs = errs;

    let (prog, tokenizer_errs) = process_tokens(tokens);
    for err in tokenizer_errs {
        errs.push(err);
    }

    debug!(?prog);

    if errs.len() > 0 {
        return Err(errs);
    }

    match (interpret, prog) {
        (true, Some(prog)) => execute(prog, &mut out),
        _ => {}
    };

    Ok(())
}

fn main() -> Result<ExitCode, Vec<String>> {
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

    run(args.filename, args.interpret, stdout()).map(|_| {
        println!("Compilation successful");
        ExitCode::SUCCESS
    })
}

fn split_errs(results: Vec<TokenParseResult>) -> (Vec<ParsedToken>, Vec<String>) {
    let errs = Vec::new();
    let mut tokens = Vec::new();
    for res in results {
        tokens.push(res.result);
    }

    (tokens, errs)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filename: String,
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
    /// Enables interpreter mode
    #[arg(short, default_value = "true")]
    interpret: bool,
}

#[cfg(test)]
mod test {
    use std::{fs, io::BufWriter, path::Path};

    use test_generator::test_resources;

    use crate::run;

    #[test_resources("tests/res/lci/test/1.3-Tests/1-Structure/**")]
    fn lci_structure_tests(resource: &str) {
        let test_dir = Path::new(resource);

        let contains_err_file = {
            let mut err_file = test_dir.to_path_buf();
            err_file.push("test.err");
            err_file.is_file()
        };

        let mut input_file = test_dir.to_path_buf();
        input_file.push("test.lol");

        let mut buf = BufWriter::new(Vec::new());
        let result = run(input_file, true, &mut buf);
        let out_str = String::from_utf8(buf.into_inner().expect("convert buf to output bytes"))
            .expect("convert output bytes to utf-8 string");

        let out_file = {
            let mut out_file = test_dir.to_path_buf();
            out_file.push("test.out");
            out_file
        };

        if out_file.is_file() {
            let out_content = fs::read_to_string(out_file).expect("Unable to read provided file");
            println!("Testing output");
            assert_eq!(
                out_content, out_str,
                "prog output does not match test output"
            );
        }

        println!("Output: {out_str}");
        assert_eq!(contains_err_file, result.is_err())
    }
}

mod parser;
mod tokenizer;

use clap::Parser;
use mediator_tracing::tracing::debug;
use mediator_tracing::TracingModule;
use parser::process_tokens;
use std::{fs, path::Path, process::ExitCode};
use tokenizer::{ParsedToken, TokenParseResult};

use crate::tokenizer::parse_tokens;

fn run<P: AsRef<Path>>(path: P) -> Result<(), Vec<String>> {
    let file_contents = fs::read_to_string(path).expect("Unable to read provided file");
    let parse_results = parse_tokens(file_contents);

    let (tokens, errs) = split_errs(parse_results);
    debug!(tokens = ?&tokens);

    let mut errs = errs;

    for err in process_tokens(tokens) {
        errs.push(err);
    }

    if errs.len() > 0 {
        Err(errs)
    } else {
        Ok(())
    }
}

fn main() -> Result<ExitCode, Vec<String>> {
    let args = Args::parse();
    TracingModule::from_level(args.log_level).init();

    run(args.filename).map(|_| {
        println!("Compilation successful");
        ExitCode::SUCCESS
    })
}

fn split_errs(results: Vec<TokenParseResult>) -> (Vec<ParsedToken>, Vec<String>) {
    let mut errs = Vec::new();
    let mut tokens = Vec::new();
    for res in results {
        match res {
            Ok(token) => tokens.push(token),
            Err(err) => errs.push(err),
        }
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
}

#[cfg(test)]
mod test {
    use std::path::Path;

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

        let result = run(input_file);

        assert_eq!(contains_err_file, result.is_err())
    }
}

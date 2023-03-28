mod parser;
mod tokenizer;

use parser::process_tokens;
use std::{fs, path::Path, process::ExitCode};
use tokenizer::{ParsedToken, TokenParseResult};

use crate::tokenizer::parse_tokens;

fn run<P: AsRef<Path>>(path: P) -> Result<(), Vec<String>> {
    let file_contents = fs::read_to_string(path).expect("Unable to read provided file");
    let parse_results = parse_tokens(file_contents);

    let (tokens, errs) = split_errs(parse_results);

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
    let filename = "tests/res/lci/test/1.3-Tests/1-Structure/5-IndentationIgnored/test.lol";

    run(filename).map(|_| {
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use test_generator::test_resources;

    use crate::run;

    #[test_resources("tests/res/lci/test/1.3-Tests/1-Structure/[1-5]-*")]
    fn lci_test(resource: &str) {
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

use std::{fs, io::stderr, path::Path};

use test_generator::test_resources;

use crate::{framework::App, Mode};

#[test_resources("tests/res/lci/test/1.3-Tests/1-Structure/**")]
fn lci_structure_tests(resource: &str) {
    run_dir(resource)
}

mod comments {
    use test_generator::test_resources;

    use super::run_dir;

    #[test_resources("tests/res/lci/test/1.3-Tests/2-Comments/1-SingleLine/**")]
    fn single_line_tests(resource: &str) {
        run_dir(resource)
    }

    #[test_resources("tests/res/lci/test/1.3-Tests/2-Comments/2-MultipleLine/**")]
    fn multiple_line_tests(resource: &str) {
        run_dir(resource)
    }
}

fn run_dir(resource: &str) {
    let test_dir = Path::new(resource);

    let contains_err_file = {
        let mut err_file = test_dir.to_path_buf();
        err_file.push("test.err");
        err_file.is_file()
    };

    let mut input_file = test_dir.to_path_buf();
    input_file.push("test.lol");

    let mut output = Vec::new();
    let result = App::new(&mut output, stderr()).run(input_file, Mode::Interpret);
    let out_str = String::from_utf8(output).expect("convert output bytes to utf-8 string");

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

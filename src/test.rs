use std::{fs, io::BufWriter, path::Path};

use test_generator::test_resources;

use crate::run;

#[test_resources("tests/res/lci/test/1.3-Tests/1-Structure/**")]
fn lci_structure_tests(resource: &str) {
    run_dir(resource)
}

#[test_resources("tests/res/lci/test/1.3-Tests/2-Comments/1-SingleLine/**")]
fn comments__single_line_tests(resource: &str) {
    run_dir(resource)
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

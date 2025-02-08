use rstest::*;
use testing::Test;
use std::path::PathBuf;
use md_latex;
#[rstest]
fn main(#[files("tests/predone/test1-*.md")] path: PathBuf) {
    println!("--{}--", path.display());
    let mut parser = Test::default_parser();
    md_latex::add(&mut parser);
    let test = Test::from_file(path).unwrap();
    let matches = test.output_matches_log(&parser);
    assert!(matches);
}

use rstest::*;
use testing::Test;
use std::path::PathBuf;
use md_toc;

#[rstest]
fn main(#[files("tests/predone/test1-*.md")] path: PathBuf) {
    let mut parser = Test::default_parser();
    md_toc::add(&mut parser);
    let test = Test::from_file(path).unwrap();
    let matches = test.output_matches_log(&parser);
    assert!(matches);
}

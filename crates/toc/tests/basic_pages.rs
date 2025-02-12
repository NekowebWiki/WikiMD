use rstest::*;
use testing::Test;
use std::path::PathBuf;
use markdown_it_table_of_contents;

#[rstest]
fn main(#[files("tests/predone/test1-*.md")] path: PathBuf) {
    let mut parser = Test::default_parser();
    markdown_it_table_of_contents::add(&mut parser);
    let test = Test::from_file(path).unwrap();
    let matches = test.output_matches_log(&parser);
    assert!(matches);
}

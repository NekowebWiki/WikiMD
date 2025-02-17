use rstest::*;
use testing::Test;
use std::path::PathBuf;
use markdown_it_footnotes;

#[rstest]
fn main(#[files("tests/predone/test1-*.md")] path: PathBuf) {
    let mut parser = Test::default_parser();
    markdown_it_footnotes::add(&mut parser);
    let test = Test::from_file(path).unwrap();
    let matches = test.output_log(&parser);
    assert_eq!(matches.0, matches.1);
}

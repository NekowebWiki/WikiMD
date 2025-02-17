use std::path::Path;
use markdown_it::MarkdownIt;
use std::convert::AsRef;

pub struct Test {
    input: String,
    expected: String,
    split: String
}

impl Test {
    pub fn from(predone: &str) -> Self {
        let mut lines = predone.lines();
        let splitting = lines.next().unwrap();
        let mut input = String::new();
        let mut expect = String::new();
        while let Some(line) = lines.next() {
            if line == splitting {
                println!("line is splitting: {}", line);
                break;
            }
            let final_line = line.to_string() + "\n";
            input.push_str(&final_line);
        }
        while let Some(line) = lines.next() {
            let final_line = line.to_string() + "\n";
            expect.push_str(&final_line);
        }

        
        Test {
            input: input,
            expected: expect,
            split: splitting.to_string()
        }
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let contents = std::fs::read_to_string(path);
        Ok(Test::from(&contents?))
    }

    pub fn default_parser() -> MarkdownIt {
        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        parser
    }

    pub fn output_matches(&self, md: &MarkdownIt) -> bool {
        md.parse(&self.input).render() == self.expected
    }
    pub fn output_matches_log(&self, md: &MarkdownIt) -> bool {
        let actual_output = md.parse(&self.input).render();
        let matches = actual_output == self.expected;
        println!("EXPECTED\n---\n{}\n---\nACTUAL\n---\n{}\n---\nMATCHES? {}", self.expected, actual_output, matches);
        matches
    }

    pub fn output(&self, md: &MarkdownIt) -> (String, String) {
        (md.parse(&self.input).render(), self.expected.clone())
    }
    pub fn output_log(&self, md: &MarkdownIt) -> (String, String) {
        let actual_output = md.parse(&self.input).render();
        let matches = self.expected == actual_output;
        println!("EXPECTED\n---\n{}\n---\nACTUAL\n---\n{}\n---\nMATCHES? {}", self.expected, actual_output, matches);
        (actual_output, self.expected.clone())
    }

    pub fn log(&self) {
        println!("{} {} {}", self.input, self.split, self.expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proper_parsing() {
        let tested = "...
# Lorem
ipsum
...
<h1>Lorem</h1>
<p>ipsum</p>";
        let test = Test::from(tested);
        let input       = "# Lorem
ipsum\n".to_string();
        let expect = "<h1>Lorem</h1>
<p>ipsum</p>\n".to_string();
        let split = "...".to_string();
        assert_eq!(
            (
                test.input,
                test.expected,
                test.split
            ),
            (
                input,
                expect,
                split
            )
        );
    }


    #[test]
    fn test_tests() {
        let parser = &mut MarkdownIt::new();
        markdown_it::plugins::cmark::add(parser);

        let tested = "...
# Lorem
ipsum
...
<h1>Lorem</h1>
<p>ipsum</p>";
        assert!(Test::from(tested).output_matches(parser));
    }
    
}

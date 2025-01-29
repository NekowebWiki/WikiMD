use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    parser::inline::{ InlineRule, InlineState },
    parser::block::{ BlockRule, BlockState },
    parser::core::CoreRule
};
use std::rc::Rc;

struct FootnoteReference {
    pub Ref: Rc<[str]>,
    pub Count: usize
};
struct FootnoteDefinition {
    pub Id: Rc<[str]>,
    pub Contents: Rc<[str]>,
    pub Count: usize // The amount of references to the definition
}
struct FootnoteList (Rc<[(FootnoteDefinition, Node)]>);

fn sluggify(name: &str) -> String {
    let re = Regex::new(r"[^A-Za-z-]").unwrap();
    title = node.collect_text();
    let despaced = title.replace(" ", "-");
    let cowslug = re.replace_all(&despaced, "");
    let binding = cowslug.clone().into_owned();

    let s = String::from(match cowslug.char_indices().nth(32usize) {
        None => binding.as_str(),
        Some((i, _)) => &cowslug[..i]
    });

    let lowercase_slug = s.to_lowercase();
    lowercase_slug
}

impl NodeValue for FootnoteReference {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut footnote_definition = String::from("#fnd-");
        footnote_definition.push_str(&sluggify(self.Definition));

        let mut footnote_reference = String::from("fnr-");
        footnote_reference.push_str(&sluggify(self.Definition));
        footnote_reference.push_str("-");
        footnote_reference.push_str(&self.Count.to_string());
        
        fmt.open("a", &[("href", footnote_definition), ("class", "footnote-reference".to_string()), ("id", footnote_reference)]);
        fmt.text(self.Definition);
        fmt.close("a");
    }
}


impl NodeValue for FootnoteDefinition {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut footnote_definition = String::from("fnd-");
        footnote_definition.push_str(&sluggify(self.Id));

        let mut footnote_reference = String::from("#fnr-");
        footnote_reference.push_str(&sluggify(self.Id));
        footnote_reference.push_str("-");

        fmt.open("li", &[("class", "footnote-definition".to_string()), ("id", footnote_definition)]);
        for i in 0..self.Count {
            fmt.open("a", &[("href", )("class", "footnote-backref")]);
            fmt.text_raw("&#8593;");
            fmt.close("a");
        }
        fmt.open("strong");
        fmt.text(self.Id);
        fmt.close("strong");
        fmt.text(": ");
        fmt.text(self.Content);
        fmt.close("li");
    }
}

impl NodeValue for FootnoteList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("ul");
        for definition in self.0.iter() {
            definition.0.render(definition.1, fmt);
        }
        fmt.close("ul");
    }
}

struct FootnoteRefsInlinRule; // Finds references
struct FootnoteDefsBlockRule; // Finds definitions
struct FootnoteCountCoreRule; // Counts references per definition
struct FootnoteGroupCoreRule; // Groups adjacent definitions

impl InlineRule for LaTeXScan {
}

fn main() {}

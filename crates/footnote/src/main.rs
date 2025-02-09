use regex::Regex;
use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    parser::{
        inline::{ InlineRule, InlineState },
        block::{ BlockRule, BlockState },
        core::CoreRule,
        extset::MarkdownItExt
    }
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct FootnoteOptions {
    FootnoteDefIdPrefix: String,
    FootnoteRefIdPrefix: String,
    
    FootnoteBackRefTxt: String,
    FootnoteBackRefCls: String,

    FootnoteDefClassName: String,
    FootnoteRefClassName: String,

    FootnoteDefListClassName: String,
}

impl Default for FootnoteOptions {
    fn default() -> Self {
        Self {
            FootnoteDefClassName: "footnotes-def".to_string(),
            FootnoteDefIdPrefix: "fnd".to_string(),

            FootnoteBackRefTxt: "&#8593;".to_string(),
            FootnoteBackRefCls: "footnote-back".to_string(),
    
            FootnoteRefIdPrefix: "fnr".to_string(),
            FootnoteRefClassName: "footnotes-ref".to_string(),

            FootnoteDefListClassName: "footnotes-list".to_string(),
        }
    }
}
impl MarkdownItExt for FootnoteOptions {}

#[derive(Debug)]
struct FootnoteReference {
    pub Ref: String,
    pub Count: usize,

    pub IdPref: String,
    pub Class: String,
    
    pub DefIdPref: String,
}
#[derive(Debug)]
struct FootnoteDefinition {
    pub Id: String,
    pub Count: usize, // The amount of references to the definition

    pub IdPref: String,
    pub Class: String,

    pub BRText: String,
    pub BRClass: String,

    pub RefIdPref: String,
}
#[derive(Debug)]
struct FootnoteList {
    pub Content: Vec<(FootnoteDefinition, Node)>,

    pub Class: String,
}

fn sluggify(name: &str) -> String {
    let re = Regex::new(r"[^A-Za-z-]").unwrap();
    let title = name;
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
        let footnote_definition = String::from("#") + &self.DefIdPref + "-" + &sluggify(&self.Ref);

        let footnote_reference = String::new() + &self.IdPref + "-" + &sluggify(&self.Ref) + "-" + &self.Count.to_string();
        
        fmt.open("a", &[("href", footnote_definition), ("class", self.Class.clone()), ("id", footnote_reference)]);
        fmt.text(&self.Ref);
        fmt.close("a");
    }
}


impl NodeValue for FootnoteDefinition {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let footnote_definition = String::new() + &self.IdPref + "-" + &sluggify(&self.Id);

        let footnote_reference = String::from("#") + &self.RefIdPref + "-" + &sluggify(&self.Id) + "-";

        fmt.open("li", &[("class", self.Class.clone()), ("id", footnote_definition)]);
        for i in 0..self.Count {
            let fn_ref = footnote_reference.clone() + &i.to_string();
            fmt.open("a", &[("href", fn_ref), ("class", self.BRClass.clone())]);
            fmt.text_raw(&self.BRText);
            fmt.close("a");
        }
        fmt.open("strong", &[]);
        fmt.text(&self.Id);
        fmt.close("strong");
        fmt.text(": ");
        fmt.contents(&node.children);
        fmt.close("li");
    }
}

impl NodeValue for FootnoteList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("ul", &[("class", self.Class.clone())]);
        for definition in self.Content.iter() {
            definition.0.render(&definition.1, fmt);
        }
        fmt.close("ul");
    }
}

struct FootnoteRefsInlinRule; // Finds references
struct FootnoteDefsBlockRule; // Finds definitions
struct FootnoteCountCoreRule; // Counts references per definition
struct FootnoteGroupCoreRule; // Groups adjacent definitions

impl InlineRule for FootnoteRefsInlinRule {
    const MARKER: char = '[';
    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..];
        if !input.starts_with("[^") || &state.src[state.pos-1..] == "\n" {
            return None;
        }
        
        let mut last_pos = 0;
        for i in state.pos+2..input.len() {
            if !input[i..].starts_with("]") || input[i-1..].starts_with(r"\") {
                continue;
            }
            last_pos = i;
            break;
        }
        let label = String::from(&input[state.pos+2..last_pos]);

        let options = state.md.ext.get::<FootnoteOptions>().unwrap();

        let ref_id_pref = options.FootnoteRefIdPrefix.clone();
        let ref_class   = options.FootnoteRefClassName.clone();

        let def_id_pref = options.FootnoteDefIdPrefix.clone();
        Some((
            Node::new(FootnoteReference {
                Ref: label,
                Count: 0,

                IdPref: ref_id_pref,
                Class: ref_class,

                DefIdPref: def_id_pref
            }),
            state.pos-last_pos
        ))
    }
}

const DEF_CONTINUE_INDENT: i32 = 4;

impl FootnoteDefsBlockRule {
    fn get_label(state: &mut BlockState) -> Option<String> {
        let line = state.get_line(state.line);
        if !line.starts_with("[^") { return None; }

        let mut character = 2;
        while character < line.len() {
            if line[character..].starts_with("]") { break; }
            character += 1;
        }
        let label = &line[2..character];

        Some(label.to_string())
    }
}
impl BlockRule for FootnoteDefsBlockRule {
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let label = Self::get_label(state)?;

        let init_line = state.line;
        state.blk_indent += 4;
        state.md.block.tokenize(state);
        state.blk_indent -= 4;
        let content_lines = state.line - init_line;

        let options = state.md.ext.get::<FootnoteOptions>().unwrap();

        let def_id_pref = options.FootnoteDefIdPrefix.clone();
        let def_class   = options.FootnoteDefClassName.clone();

        let br_text     = options.FootnoteBackRefTxt.clone();
        let br_class    = options.FootnoteBackRefCls.clone();

        let ref_id_pref = options.FootnoteRefIdPrefix.clone();
        Some((
            Node::new(FootnoteDefinition {
                Id: label,
                Count: 0,

                IdPref: def_id_pref,
                Class: def_class,

                BRText: br_text,
                BRClass: br_class,

                RefIdPref: ref_id_pref
            }),
            content_lines
        ))
    }
}

impl CoreRule for FootnoteCountCoreRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let mut counts: HashMap<String, usize> = HashMap::new();

        root.walk_mut(|node, _| {
            let mut reference = match node.cast_mut::<FootnoteReference>() {
                Some(r) => r,
                None    => return ()
            };
            let ref_id = reference.Ref.clone();
            counts.entry(ref_id.clone()).and_modify(|c| *c += 1).or_insert(1);
            let count = counts[&ref_id];
            reference.Count = count;
        });
        root.walk_mut(|node, _| {
            let mut definition = match node.cast_mut::<FootnoteDefinition>() {
                Some(r) => r,
                None    => return ()
            };
            let def_id = definition.Id.clone();
            counts.entry(def_id.clone()).or_insert(0);
            let count = counts[&def_id];
            definition.Count = count;
        });
    }
}

impl CoreRule for FootnoteGroupCoreRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let options = md.ext.get::<FootnoteOptions>().unwrap();
        
        let deflist_class = options.FootnoteDefListClassName.clone();
    }
}

fn main() {}

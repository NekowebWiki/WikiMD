use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    parser::{
        inline::{ InlineRule, InlineState },
        block::{ BlockRule, BlockState },
        core::CoreRule,
        extset::MarkdownItExt
    },
    plugins::cmark::block::reference::ReferenceScanner
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
}
#[derive(Debug)]
struct FootnoteDefinition {
    pub Id: String,
    pub Count: usize, // The amount of references to the definition

    pub BRText: String,
    pub BRClass: String,

    pub RefIdPref: String,
}
#[derive(Debug)]
struct FootnoteList(usize);

fn sluggify(name: &str) -> String {
    name.to_string().replace(|c| !char::is_alphanumeric(c) && c != ' ', "").replace(" ", "-").to_lowercase()
}

impl NodeValue for FootnoteReference {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("a", &node.attrs);
        fmt.text(&self.Ref);
        fmt.close("a");
    }
}


impl NodeValue for FootnoteDefinition {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("li", &node.attrs);
        fmt.cr();
        for i in 0..self.Count {
            let fn_ref = self.RefIdPref.clone() + &i.to_string();
            fmt.open("a", &[("href", fn_ref), ("class", self.BRClass.clone())]);
            fmt.text_raw(&self.BRText);
            fmt.close("a");
        }
        fmt.open("strong", &[]);
        fmt.text(&self.Id);
        fmt.close("strong");
        fmt.text(": ");
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("li");
        fmt.cr();
    }
}

impl NodeValue for FootnoteList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("ul", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
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
        for i in 2..input.len() {
            if !input[i..].starts_with("]") || input[i-1..].starts_with(r"\") {
                continue;
            }
            last_pos = i;
            break;
        }
        let label = String::from(&input[2..last_pos]);

        let options = state.md.ext.get::<FootnoteOptions>().unwrap();

        let ref_class = options.FootnoteRefClassName.clone();

        let def_id_pref = options.FootnoteDefIdPrefix.clone();
        let mut node = Node::new(FootnoteReference {
            Ref: label.clone(),
            Count: 0,
        });

        node.attrs.push(("class", ref_class));
        node.attrs.push(("href", String::from("#") + &def_id_pref + "-" + &sluggify(&label)));
        Some((
            node,
            last_pos+1
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

        let options = state.md.ext.get::<FootnoteOptions>().unwrap();

        let def_id_pref = options.FootnoteDefIdPrefix.clone();
        let def_class   = options.FootnoteDefClassName.clone();

        let br_text     = options.FootnoteBackRefTxt.clone();
        let br_class    = options.FootnoteBackRefCls.clone();

        let ref_id_pref = options.FootnoteRefIdPrefix.clone();
        
        let mut node = Node::new(FootnoteDefinition {
            Id: label.clone(),
            Count: 0,

            BRText: br_text,
            BRClass: br_class,

            RefIdPref: ref_id_pref + "-" + &sluggify(&label) + "-"
        }); 

        node.attrs.push(("id", def_id_pref + "-" + &sluggify(&label)));
        node.attrs.push(("class", def_class));

        let old_node = std::mem::replace(&mut state.node, node);

        let init_line = state.line;
        let init_offsets = state.line_offsets[init_line].clone();

        state.line_offsets[init_line].first_nonspace  += 5 + label.clone().len();
        state.line_offsets[init_line].indent_nonspace += 5i32;
        
        state.blk_indent += 4;
        state.md.block.tokenize(state);
        state.blk_indent -= 4;
        
        let content_lines = state.line - init_line;
        
        state.line = init_line;
        state.line_offsets[init_line] = init_offsets; 
        
        Some((
            std::mem::replace(&mut state.node, old_node),
            content_lines
        ))
    }
}

impl CoreRule for FootnoteCountCoreRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let mut counts: HashMap<String, usize> = HashMap::new();

        let options = md.ext.get::<FootnoteOptions>().unwrap();
        let footnote_reference  = |id: &str, count: usize| options.FootnoteRefIdPrefix.clone() + "-" + &sluggify(id) + "-" + &count.to_string();
        root.walk_mut(|node, _| {
            let reference = match node.cast_mut::<FootnoteReference>() {
                Some(r) => r,
                None    => return ()
            };
            let ref_id = reference.Ref.clone();
            counts.entry(ref_id.clone()).and_modify(|c| *c += 1).or_insert(1);
            let count = counts[&ref_id];
            reference.Count = count;

            node.attrs.push(("id", footnote_reference(&ref_id, count)));
        });
        root.walk_mut(|node, _| {
            let definition = match node.cast_mut::<FootnoteDefinition>() {
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

#[derive(Debug)]
struct PlaceholderNode();
impl NodeValue for PlaceholderNode {}

impl CoreRule for FootnoteGroupCoreRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let mut index = 0;
        let mut open_index = 0;
        let options = md.ext.get::<FootnoteOptions>().unwrap();
        let class = options.FootnoteDefListClassName.clone();
        let mut defs: Vec<(usize, Box<Node>)> = Vec::new();
        root.walk_mut(|node, _depth| {
            if !node.is::<FootnoteDefinition>() {
                index += 1;
                open_index = index;
                return;
            }
            let mut replaced = if index != open_index { Node::new(PlaceholderNode()) } else { Node::new(FootnoteList(open_index)) };
            replaced.attrs.push(("class", class.clone()));
            let extract = std::mem::replace(node, replaced);
            defs.push((open_index, Box::new(extract)));
        });
        index = 0;
        root.children.retain(|child| {
            if !child.is::<PlaceholderNode>() {
                return true;
            }
            index += 1;
            false
        });
        root.walk_mut(|node, _depth| {
            let def_list_id = match node.cast_mut::<FootnoteList>() {
                Some(FootnoteList(n)) => *n,
                None => return (),
            };
            for def in &mut defs {
                if def.0 != def_list_id { continue; }
                let a = std::mem::replace(&mut def.1, Box::new(Node::new(PlaceholderNode())));
                node.children.push(*a);
            }
        });
    }
}

fn add(md: &mut MarkdownIt) {
    md.ext.get_or_insert_default::<FootnoteOptions>();
    md.inline.add_rule::<FootnoteRefsInlinRule>();
    md.block.add_rule::<FootnoteDefsBlockRule>().before::<ReferenceScanner>();
    md.add_rule::<FootnoteCountCoreRule>();
    md.add_rule::<FootnoteGroupCoreRule>().after::<FootnoteCountCoreRule>();
}

fn add_with_options(md: &mut MarkdownIt, options: FootnoteOptions) {
    md.ext.insert(options);
    md.inline.add_rule::<FootnoteRefsInlinRule>();
    md.block.add_rule::<FootnoteDefsBlockRule>().before::<ReferenceScanner>();
    md.add_rule::<FootnoteCountCoreRule>();
    md.add_rule::<FootnoteGroupCoreRule>().after::<FootnoteCountCoreRule>();
}

fn main() {
    let markdown = r#"Hello![^foo]

[^foo]: bar!
    Newline or *something*
"#;
    let parser = &mut MarkdownIt::new();
    markdown_it::plugins::cmark::add(parser);
    add(parser);
    let testing = parser.parse(markdown).render();
    println!("{}", testing);
}

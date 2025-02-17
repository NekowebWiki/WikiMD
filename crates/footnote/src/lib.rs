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
    fn_def_id_pref: String,
    fn_ref_id_pref: String,
    
    fn_br_text: String,
    fn_br_class: String,

    fn_def_class: String,
    fn_ref_class: String,

    fn_list_class: String,
}

impl Default for FootnoteOptions {
    fn default() -> Self {
        Self {
            fn_def_class: "footnotes-def".to_string(),
            fn_def_id_pref: "fnd".to_string(),

            fn_br_text: "&#8593;".to_string(),
            fn_br_class: "footnote-back".to_string(),
    
            fn_ref_id_pref: "fnr".to_string(),
            fn_ref_class: "footnotes-ref".to_string(),

            fn_list_class: "footnotes-list".to_string(),
        }
    }
}
impl MarkdownItExt for FootnoteOptions {}

#[derive(Debug)]
struct FootnoteReference {
    pub r#ref: String,
    pub count: usize,
}
#[derive(Debug)]
struct FootnoteDefinition {
    pub id: String,
    pub count: usize, // The amount of references to the definition

    pub br_text: String,
    pub br_class: String,

    pub ref_id_prefix: String,
}
#[derive(Debug)]
struct FootnoteList(usize);

fn sluggify(name: &str) -> String {
    name.to_string().replace(|c| !char::is_alphanumeric(c) && c != ' ', "").replace(" ", "-").to_lowercase()
}

impl NodeValue for FootnoteReference {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("a", &node.attrs);
        fmt.text(&self.r#ref);
        fmt.close("a");
    }
}


impl NodeValue for FootnoteDefinition {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("li", &node.attrs);
        fmt.cr();
        for i in 0..self.count {
            let fn_i = (i+1).to_string();
            let fn_ref = self.ref_id_prefix.clone() + &fn_i;
            fmt.open("a", &[("href", fn_ref), ("class", self.br_class.clone())]);
            fmt.text_raw(&self.br_text);
            fmt.close("a");
            fmt.cr();
        }
        fmt.open("strong", &[]);
        fmt.text(&self.id);
        fmt.close("strong");
        fmt.text(":");
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("li");
        fmt.cr();
    }
}

impl NodeValue for FootnoteList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("ul", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.close("ul");
        fmt.cr();
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

        let ref_class = options.fn_ref_class.clone();

        let def_id_pref = options.fn_def_id_pref.clone();
        let mut node = Node::new(FootnoteReference {
            r#ref: label.clone(),
            count: 0,
        });

        node.attrs.push(("class", ref_class));
        node.attrs.push(("href", String::from("#") + &def_id_pref + "-" + &sluggify(&label)));
        Some((
            node,
            last_pos+1
        ))
    }
}

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

        let def_id_pref = options.fn_def_id_pref.clone();
        let def_class   = options.fn_def_class.clone();

        let br_text     = options.fn_br_text.clone();
        let br_class    = options.fn_br_class.clone();

        let ref_id_pref = options.fn_ref_id_pref.clone();
        
        let mut node = Node::new(FootnoteDefinition {
            id: label.clone(),
            count: 0,

            br_text: br_text,
            br_class: br_class,

            ref_id_prefix: ref_id_pref + "-" + &sluggify(&label) + "-"
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
        let footnote_reference  = |id: &str, count: usize| options.fn_ref_id_pref.clone() + "-" + &sluggify(id) + "-" + &count.to_string();
        root.walk_mut(|node, _| {
            let reference = match node.cast_mut::<FootnoteReference>() {
                Some(r) => r,
                None    => return ()
            };
            let ref_id = reference.r#ref.clone();
            counts.entry(ref_id.clone()).and_modify(|c| *c += 1).or_insert(1);
            let count = counts[&ref_id];
            reference.count = count;

            node.attrs.push(("id", footnote_reference(&ref_id, count)));
        });
        root.walk_mut(|node, _| {
            let definition = match node.cast_mut::<FootnoteDefinition>() {
                Some(r) => r,
                None    => return ()
            };
            let def_id = definition.id.clone();
            counts.entry(def_id.clone()).or_insert(1);
            let count = counts[&def_id];
            definition.count = count;

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
        let class = options.fn_list_class.clone();
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

pub fn add(md: &mut MarkdownIt) {
    md.ext.get_or_insert_default::<FootnoteOptions>();
    md.inline.add_rule::<FootnoteRefsInlinRule>();
    md.block.add_rule::<FootnoteDefsBlockRule>().before::<ReferenceScanner>();
    md.add_rule::<FootnoteCountCoreRule>();
    md.add_rule::<FootnoteGroupCoreRule>().after::<FootnoteCountCoreRule>();
}

pub fn add_with_options(md: &mut MarkdownIt, options: FootnoteOptions) {
    md.ext.insert(options);
    md.inline.add_rule::<FootnoteRefsInlinRule>();
    md.block.add_rule::<FootnoteDefsBlockRule>().before::<ReferenceScanner>();
    md.add_rule::<FootnoteCountCoreRule>();
    md.add_rule::<FootnoteGroupCoreRule>().after::<FootnoteCountCoreRule>();
}


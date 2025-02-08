mod latex;
use crate::latex::latex_render_mathml;

use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    parser::inline::{ InlineRule, InlineState }
};

#[derive(Debug)]
pub struct LaTeXNode {
    latex: String,
    inline: bool
}

impl NodeValue for LaTeXNode {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mathml = latex_render_mathml(&self.latex, self.inline);
        fmt.text_raw(&mathml);
    }
}

struct LaTeXScan;

impl InlineRule for LaTeXScan {
    const MARKER: char = '$';
    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..];
        if !input.starts_with("$") { return None; }
        
        let latex_enter_count = if input.starts_with("$$") { 2 } else { 1 };

        let start_pos = state.pos + latex_enter_count;
        let mut last_posi = state.pos + latex_enter_count;
        let mut next_posi = state.pos + latex_enter_count;

        while next_posi < state.pos_max {
            let current_input = &state.src[next_posi..];
            if !current_input.starts_with(&"$".repeat(latex_enter_count)) {
                next_posi += 1;
                continue;
            }

            last_posi = next_posi;
            break;
        }
        let latex_text = &state.src[start_pos..last_posi];
        Some((
            Node::new(LaTeXNode {
                latex: latex_text.to_string(),
                inline: latex_enter_count = 1
            }),
            last_posi + latex_enter_count - (state.pos)
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<LaTeXScan>();
}

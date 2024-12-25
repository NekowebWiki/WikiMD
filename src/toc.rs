use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    plugins::cmark::block::{
        heading::ATXHeading,
        lheading::SetextHeader
    },
    parser::core::CoreRule
};
use regex::Regex;
use std::vec::Vec;
use unbox_box::BoxExt;

#[derive(Debug)]
pub struct TableOfContentsItem {
    pub slug: String,
    pub title: String,
    pub children: Box<Vec<TableOfContentsItem>>
}

impl TableOfContentsItem {
    fn push(&mut self, item: TableOfContentsItem, level: u8) {
        let child_count = self.children.unbox_ref().len();
        if level > 0 && child_count != 0 {
            self.children.unbox_mut()[child_count - 1].push(item, level - 1);
            return;
        }
        self.children.unbox_mut().push(item);
    }
}

#[derive(Debug)]
pub struct TOC(Vec<TableOfContentsItem>);

impl NodeValue for TOC {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {

        fn render_item(item: &TableOfContentsItem, fmt: &mut dyn Renderer) {
            fmt.open("li", &[]);

            let mut link_href = String::from("#");
            link_href.push_str(&item.slug);
            fmt.open("a", &[("href", link_href)]);
            fmt.text(&item.title);
            fmt.close("a");
            if item.children.len() != 0 {
                fmt.open("ol", &[]);
                item.children.unbox_ref().iter().for_each(|child| {
                    render_item(&(child), fmt);
                });
                fmt.close("ol");
            }
            fmt.close("li");
        }
        let attrs = node.attrs.clone();
        
        fmt.cr();
        fmt.open("ol", &attrs);
        self.0.iter().for_each(|item| {
            render_item(item, fmt);
        });
        fmt.close("ol");
        fmt.cr();
    }
}

impl TOC {
    fn push(&mut self, item: TableOfContentsItem, level: u8) {
        let child_count = self.0.len();
        if level > 0 && child_count != 0 {
            self.0[child_count - 1].push(item, level - 1);
            return;
        }
        self.0.push(item);
    }
}


struct TableOfContentsDetect;

impl CoreRule for TableOfContentsDetect {
    fn run(root: &mut Node, _: &MarkdownIt) {
        struct Heading {
            level: u8,
            title: String,
            slug: String
        }

        let mut disorganized_headings: Vec<Heading> = Vec::new();
        root.walk_mut(|node, _| {
            let nodetype = node.node_type.name;

            fn get_level(node: &Node) -> Option<u8> {
                match node.cast::<ATXHeading>() {
                    Some(item) => return Some(item.level),
                    None => ()
                };
                match node.cast::<SetextHeader>() {
                    Some(item) => return Some(item.level),
                    None => ()
                };
                return None
            }

            let level = match get_level(node) {
                None => return (),
                Some(l) => l
            };

            let re = Regex::new(r"[^A-Za-z-]").unwrap();
            let title = node.collect_text();
            let despaced = title.replace(" ", "-");
            let cowslug = re.replace_all(&despaced, "");
            let binding = cowslug.clone().into_owned();
            let slug = match cowslug.char_indices().nth(32usize) {
                None => binding.as_str(),
                Some((i, _)) => &cowslug[..i]
            };

            let mut attrs = node.attrs.clone();
            attrs.push(("id", slug.into()));
            node.attrs = attrs;

            let header_tag = Heading {
                title: title,
                slug: String::from(slug),
                level: level
            };
            disorganized_headings.push(header_tag);
        });

        let mut organized_headings: TOC = TOC{ 0: Vec::new() };
        for heading in disorganized_headings {
            let head = TableOfContentsItem {
                title: heading.title,
                slug: heading.slug,
                children: Box::new(Vec::new())
            };
            organized_headings.push(head, heading.level);
        }

        root.children.push(Node::new(organized_headings));
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into parser
    md.add_rule::<TableOfContentsDetect>();
}

use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    plugins::cmark::block::{
        heading::ATXHeading,
        lheading::SetextHeader
    },
    parser::core::CoreRule
};
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


fn sluggify(name: &str) -> String {
    name.to_string().replace(|c| !char::is_alphanumeric(c) && c != ' ', "").replace(" ", "-").to_lowercase()
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
        let mut index = 0;
        let mut head_count = 0;
        let mut first_heading: Option<u16> = None;
        root.walk_post_mut(|node, _| {
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

            index = index + 1;

            let level = match get_level(node) {
                None => return (),
                Some(l) => l
            };

            if first_heading == None {
                first_heading = Some((index - 1) >> 1);
                // MD-It has been adding a skip text between each element for some reason.
            }
            head_count = head_count + 1;

            let title = node.collect_text();
            let slug = match node.attrs.as_slice() {
                [("id", id)] => String::from(id),
                // If another plugin sets the id, use that instead.
                _ => {
                    let slug = sluggify(&title);
                    node.attrs.push(("id", slug.clone()));
                    slug
                }
            };
            
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

        if head_count < 2 {
            return ();
        }

        match first_heading {
            Some(i) => root.children.insert(i.into(), Node::new(organized_headings)),
            None => ()
        };
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into parser
    md.add_rule::<TableOfContentsDetect>();
}

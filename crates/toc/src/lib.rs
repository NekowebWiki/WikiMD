use markdown_it::{
    MarkdownIt, Node, NodeValue, Renderer,
    plugins::cmark::block::{
        heading::ATXHeading,
        lheading::SetextHeader,
    },
    parser::{
        core::CoreRule,
        extset::MarkdownItExt
    }
};
use std::vec::Vec;
use unbox_box::BoxExt;

#[derive(Debug)]
pub struct TableOfContentsItem {
    pub slug: String,
    pub title: String,
    pub level: u8,
    pub children: Box<Vec<TableOfContentsItem>>
}

#[derive(Debug)]
pub struct TOCOptions {
    pub allow_titles_in_toc: bool, // parse in title (h1)
    pub treat_title_as_h2: bool, // only matters if allow_titles_in_toc is true
    pub toc_class: String,
    pub wrap_in_nav: bool, // whether to wrap the table of contents in a <nav> element.
                           // If true, toc_class is applied to <nav> instead of <ol>.
    pub toc_heading: Option<( u8, String )> // level of heading to use, recommended is 2, followed
                                            // by the text in the title.
}

impl MarkdownItExt for TOCOptions {}
impl Default for TOCOptions {
    fn default() -> Self {
        TOCOptions {
            allow_titles_in_toc: false,
            treat_title_as_h2: false,
            toc_class: "table_of_contents".to_string(),
            wrap_in_nav: true,
            toc_heading: Some((2, "Contents".to_string()))
        }
    }
}

impl TableOfContentsItem {
    fn push(&mut self, item: TableOfContentsItem, depth_limit: u8) {
        let child_count = self.children.unbox_ref().len();
        if depth_limit > self.level+1 && child_count != 0 {
            self.children.unbox_mut()[child_count - 1].push(item, depth_limit-1);
            return;
        }
        self.children.unbox_mut().push(item);
    }
}

#[derive(Debug)]
pub struct TOC {
    contents: Vec<TableOfContentsItem>,
    min_level: u8,
    wrap_in_nav: bool,
    toc_heading: Option<( u8, String )>
}

impl NodeValue for TOC {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {

        fn render_item(item: &TableOfContentsItem, fmt: &mut dyn Renderer) {
            fmt.cr();
            fmt.open("li", &[]);

            let mut link_href = String::from("#");
            link_href.push_str(&item.slug);
            fmt.cr();
            fmt.open("a", &[("href", link_href)]);
            fmt.text(&item.title);
            fmt.close("a");
            if item.children.len() != 0 {
                fmt.cr();
                fmt.open("ol", &[]);
                item.children.unbox_ref().iter().for_each(|child| {
                    render_item(&(child), fmt);
                });
                fmt.cr();
                fmt.close("ol");
            }
            fmt.cr();
            fmt.close("li");
        }
        let attrs = node.attrs.clone();

        fmt.cr();
        if self.wrap_in_nav {
            fmt.open("nav", &attrs);
            fmt.cr();
        }
        if let Some(heading) = &self.toc_heading {
            let heading_tag = String::from("h") + &heading.0.to_string();
            fmt.open(&heading_tag, &[]);
            fmt.text(&heading.1);
            fmt.close(&heading_tag);
            fmt.cr();
        }
        fmt.open(
            "ol",
            if !self.wrap_in_nav { &attrs } else { &[] }
        );
        self.contents.iter().for_each(|item| {
            render_item(item, fmt);
        });
        fmt.cr();
        fmt.close("ol");
        if self.wrap_in_nav {
            fmt.cr();
            fmt.close("nav");
        }
        fmt.cr();
    }
}

impl TOC {
    fn push(&mut self, item: TableOfContentsItem, depth_limit: u8) {
        println!("{}", depth_limit);
        let child_count = self.contents.len();
        if depth_limit > self.min_level && child_count != 0 {
            self.contents[child_count - 1].push(item, depth_limit - 1);
            return;
        }
        println!("---");
        self.min_level = item.level;
        self.contents.push(item);
    }
}


fn sluggify(name: &str) -> String {
    name.to_string().replace(|c| !char::is_alphanumeric(c) && c != ' ', "").replace(" ", "-").to_lowercase()
}

struct TableOfContentsDetect;

impl CoreRule for TableOfContentsDetect {
    fn run(root: &mut Node, md: &MarkdownIt) {
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

            index += 1;

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

        let default_opts = TOCOptions::default();
        let opts = md.ext.get::<TOCOptions>().unwrap_or(&default_opts);

        let mut organized_headings = TOC {
            contents: Vec::new(),
            min_level: 6,
            wrap_in_nav: opts.wrap_in_nav,
            toc_heading: opts.toc_heading.clone()
        };
        for heading in disorganized_headings {
            let head = TableOfContentsItem {
                title: heading.title,
                slug: heading.slug,
                level: heading.level,
                children: Box::new(Vec::new())
            };
            if heading.level == 1 && !opts.allow_titles_in_toc { continue; }
            organized_headings.push(
                head,
                if heading.level == 1 && opts.treat_title_as_h2 { 2 } else { heading.level }
            );
        }

        if head_count < 2 {
            return ();
        }

        let mut table_of_contents = Node::new(organized_headings);
        table_of_contents.attrs.push(("class", opts.toc_class.clone()));
        match first_heading {
            Some(i) => root.children.insert(i.into(), table_of_contents),
            None => ()
        };
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into parser
    md.add_rule::<TableOfContentsDetect>();
}

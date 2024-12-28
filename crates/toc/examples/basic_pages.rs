extern crate md_toc;

fn main() {
    let not_enough_headings = "This page has 1 heading. (2 are required.)
## See?
Because there's not enough headings, the table of contents doesn't get added to the page.";
    let enough_headings = "This page has enough headings for the TOC to show up.

## See?

Because there's enough headings, the table appears before the first heading.

## Lorem?

Ipsum.

## Another heading

Yeah

## Where does it show up?

It shows up before the first heading.";
    let no_headings = "This is a very basic page that has 0 headings.";
    let parser = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(parser);
    md_toc::add(parser);

    println!("not_enough_headings
{}", parser.parse(not_enough_headings).render());
    println!("enough_headings
{}", parser.parse(enough_headings).render());
    println!("no_headings
{}", parser.parse(no_headings).render());
}

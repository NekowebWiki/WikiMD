# markdown-it-table-of-contents

Plugin for [markdown-it.rs](https://crates.io/crates/markdown-it) which adds a table of contents to the page.

Note that this library does generate its own ids for your headers, but only if there is not already a present id.

## Options

The following options are available under the `TOCOptions` object; however, the defaults are recommended.

```rust
TOCOptions {
    allow_titles_in_toc: false,                    // Include title (h1) in table of contents.
    treat_title_as_h2: false,                      // Whether to treat the title as an heading element (h2).
    toc_class: "table_of_contents".to_string(),    // Class given to the table of contents.
                                                   // If wrap_in_nav is true, the class is applied to the nav
                                                   // element, otherwise it's applied to the list (ol) element.
    wrap_in_nav: true,                             // If the table of contents should be wrapped in a nav element
    toc_heading: Some((2, "Contents".to_string())) // Heading placed at the top of the table of contents (but within
                                                   // nav if wrap_in_nav is true).
}
```

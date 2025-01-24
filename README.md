# WikiMD
Plugins for [markdown-it.rs](https://crates.io/crates/markdown-it), a Rust port of the JavaScript library. The goal is to reimplement extensions from [markdown2](https://pypi.org/project/markdown2/) which are used in the Nekoweb Wiki.

Note: none of these are available on crates.io, at least not yet. Might be at some point, though.

## Plugins

- [x] Table of contents: available under crates/toc
- [x] LaTeX: See [LaTeX](#latex)

#### Possibly unnecessary 
- [ ] Footnotes: [already an implementation](https://crates.io/crates/markdown-it-footnote), but I'm not entirely a fan of the look of the footnotes section

### Unnecessary
- Cuddled lists: this was fixing a limitation of the Markdown2 parser.
- Header ids: implemented in the table of contents crate, is also provided with markdown-it.rs
- fenced code blocks: markdown-it.rs seems to already feature codeblocks and syntax highlighting with [syntect](https://crates.io/crates/syntect)
- Markdown in HTML: Markdown-it.rs doesn't feature the limitation that Markdown must not be in HTML.
- strikethrough: provided already
- GFM tables: provided already
- link patterns: provided, and doesnt require a messy RegExp, too.
- Metadata: [Someone else has already implemented it](https://crates.io/crates/markdown-it-front-matter).

### Additionally
These weren't in the original set of plugins, but might be nice to add.

- https://crates.io/crates/markdown-it-lazyload
- Super- and subscript

## LaTeX

There is a [LaTeX2MathML](https://crates.io/crates/latex2mathml) crate, but it hasn't been updated in 5 years, and renders some things incorrectly. As an example: `\alpha_{2}^{4}` is rendered to `<msub><mi>α</mi><msup><mn>2</mn><mn>4</mn></msup></msub>`, but should be `<msubsup><mi>α</mi><mn>2</mn><mn>4</mn></msubsup>`; this specific example if from an open issue: [osanshouo/latex2mathml#10](https://github.com/osanshouo/latex2mathml/issues/10). Basically, a fork would be in order, but I don't know if it'd be feasible to do such.

Another crate, [Pulldown LaTeX](https://crates.io/crates/pulldown-latex), is available which appears to be able to parse much of what you can do in LaTex so far. It hasn't been updated in 2 months, but that's far better than the previous library (5 years > 2 months).

A third option is [ReTeX](https://github.com/ReTeX/ReX/tree/master). Similar to LateX2MathML, it hasn't been updated in five years. Still seems like a good option (to an extent, at least).

### Other crates

- [KaTeX.rs](https://crates.io/crates/katex) or others that rely on JavaScript: I don't really wanna rely on JavaScript.
- [MiTeX](https://github.com/mitex-rs/mitex): can't find good documentation for usage as a Rust library.
- [LaTeXML](https://github.com/dginev/latexml-runner): doesn't provide any docs, as far as I can tell. Hasn't been updated in 4 years.

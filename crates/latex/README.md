# Markdown-It-Math
A plugin for [markdown-it.rs](https://crates.io/crates/markdown-it) that adds LaTeX support powered by [Pulldown-LaTeX](https://crates.io/crates/pulldown-latex).

## Syntax
Inline math is defined using a single dollar sign (`$`), whereas block math is defined with 2 dollar signs (`$$`).

Essentially, `$\text{ this renders as an inline equation, with display equal to inline }$`, whereas `$$\text{ this renders as an block equation, with display equal to block; so it renders on its own line }$$`.

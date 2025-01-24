use pulldown_latex::{
    mathml::push_mathml,
    Parser,
    Storage,
    RenderConfig,
    config
};

pub fn latex_render_mathml(latex: &str, inline: bool) -> String {
    // Function powering the entire module. Just uses Pulldown-LaTeX.
    let store = Storage::new();
    let parse = Parser::new(latex, &store);

    let mut mathml = String::new();
    match push_mathml(
        &mut mathml, parse,
        RenderConfig {
            display_mode: if inline { config::DisplayMode::Inline } else { config::DisplayMode::Block },
            math_style: config::MathStyle::TeX,
            annotation: Some(latex),
            error_color: ( 255, 0, 127 ),
            xml: false
        }
    ) {
        Err(e) => panic!("An error happened: {}", e),
        // Taking a look at the Pulldown-LaTeX source, this function already panics if it errors.
        _ => mathml
    }
}

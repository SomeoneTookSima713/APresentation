const MARKDOWN: &'static str = r#"
* Item 1
* Item 2
* **Third** item with *italics* and **boldness**
* Fourth item with `code`
"#;

fn main() -> anyhow::Result<()> {
    init_logger();

    apresentation::presentation::renderable::text::markdown::Parser::parse(MARKDOWN);

    pollster::block_on(apresentation::run())
}

fn init_logger() {
    use log::LevelFilter;

    const MODULE_LEVELS: &[(&str, LevelFilter)] = &[
        ("apresentation", LevelFilter::Debug)
    ];

    let mut builder = pretty_env_logger::formatted_builder();
    builder.format_indent(Some(4));

    for (module, level) in MODULE_LEVELS.iter() {
        builder.filter_module(module, *level);
    }

    builder.init();
}
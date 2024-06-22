fn main() -> anyhow::Result<()> {
    init_logger();

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
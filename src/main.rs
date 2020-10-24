use {
    adbook::cli::Cli,
    anyhow::*,
    clap::Clap,
    fern::colors::{Color, ColoredLevelConfig},
};

fn main() -> Result<()> {
    self::configure_log()?;
    Cli::parse().run()
}

/// Sets up [`fern`]. Handlebars debug/trace logs are ignored (that's why `fern` is used!)
fn configure_log() -> Result<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::BrightBlack);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {} {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        // .level(log::LevelFilter::Debug)
        .level_for("handlebars", log::LevelFilter::Info)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

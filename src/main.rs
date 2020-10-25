//! `adbook`
//!
//! # TODOs
//!
//! * refactor book loading
//! * setup convert sub command: consider if the file is in site directory or not

use {
    adbook::cli::Cli,
    anyhow::*,
    clap::Clap,
    fern::colors::{Color, ColoredLevelConfig},
};

fn main() -> Result<()> {
    self::configure_log().context("Unable to condifure `adbook` logging system (`fern`)")?;
    Cli::parse().run()
}

/// Sets up [`fern`]
///
/// * ignore logs from some crates
/// * output logs to `stderr`
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
        .level_for("handlebars", log::LevelFilter::Info)
        .level_for("async_std", log::LevelFilter::Debug)
        .level_for("async_io", log::LevelFilter::Debug)
        .level_for("polling", log::LevelFilter::Debug)
        .chain(std::io::stderr())
        // .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

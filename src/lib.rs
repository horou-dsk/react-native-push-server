pub mod router;
pub mod service;

use fern::colors::{Color, ColoredLevelConfig};

pub fn setup_logger() -> Result<(), fern::InitError> {
    // let colors = ColoredLevelConfig::new().debug(Color::Magenta);
    let log_level = std::env::var("RUST_LOG");
    let log_level = match log_level {
        Ok(level) => {
            match level.as_str() {
                "info" => log::LevelFilter::Info,
                "error" => log::LevelFilter::Error,
                _ => log::LevelFilter::Debug,
            }
        },
        Err(_) => log::LevelFilter::Debug
    };
    let colors = ColoredLevelConfig::new().error(Color::Red);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}:{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.line().unwrap_or(0),
                colors.color(record.level()),
                message
            ))
        })
        .level(log_level)
        .chain(fern::Output::call(|record| {
            if record.level() == log::Level::Error {
                eprintln!("{}", record.args());
            }
        }))
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
use std::path::PathBuf;

use serde_derive::Deserialize;
use typemap::Key;

use fern::colors::{Color, ColoredLevelConfig};

use chrono;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub chain_storage_dir: PathBuf,

    #[serde(default = "default_prefix")]
    pub prefix: String,

    #[serde(default)]
    pub generation: GenerationParams,
}

impl Key for Config {
    type Value = Self;
}

#[derive(Deserialize)]
pub struct GenerationParams {
    #[serde(default)]
    pub min_words: usize,

    #[serde(default)]
    pub include_tag_only: bool,
}

impl Default for GenerationParams {
    fn default() -> Self {
        GenerationParams {
            min_words: 1,
            include_tag_only: true,
        }
    }
}

fn default_prefix() -> String {
    String::from("|")
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("serenity", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .chain(
            fern::Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{line_color}{m}{line_color}",
                        line_color =
                            format!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str()),
                        m = message
                    ));
                })
                .chain(std::io::stdout()),
        )
        .chain(fern::log_file("bizarro.log")?)
        .apply()?;
    Ok(())
}

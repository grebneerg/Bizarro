use std::{path::PathBuf};

use serde_derive::Deserialize;
use typemap::Key;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub chain_storage_dir: PathBuf,

    #[serde(default = "default_prefix")]
    pub prefix: String,

    #[serde(default)]
    pub generation: GenerationParams
}

impl Key for Config {
    type Value = Self;
}

#[derive(Deserialize)]
pub struct GenerationParams {
    pub min_words: usize,
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
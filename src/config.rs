use crate::events::events::Result;
use std::collections::hash_map::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::*;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct Config {
    pub git: HashMap<String, crate::events::git::GitConfig>,
    pub email: HashMap<String, crate::events::email::EmailConfig>,
    pub ical: HashMap<String, crate::events::ical::IcalConfig>,
}

pub fn config_path() -> Result<PathBuf> {
    let config_folder = config_folder()?;
    Ok(config_folder.join("config.toml"))
}

pub fn default_config() -> Config {
    Config {
        git: HashMap::new(),
        email: HashMap::new(),
        ical: HashMap::new(),
    }
}

pub fn read_config() -> Result<Config> {
    let config_file = config_path()?;
    if !config_file.is_file() {
        return Ok(default_config());
    }
    let mut contents = String::new();
    File::open(config_file)?.read_to_string(&mut contents)?;
    toml::from_str(&contents).map_err(|e| {
        // TODO verbose.. https://www.reddit.com/r/rust/comments/esueur/returning_trait_objects/
        Box::new(e) as Box<dyn error::Error>
    })
}

pub fn save_config(config: &Config) -> Result<()> {
    let mut file = File::create(config_path()?)?;
    let r = file.write_all(toml::to_string_pretty(config)?.as_bytes())?;
    Ok(r)
}

pub fn config_folder() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().expect("Can't find your home folder?");
    let config_folder = home_dir.join(".cigale");
    if !config_folder.is_dir() {
        fs::create_dir(&config_folder)?;
    }
    Ok(config_folder)
}

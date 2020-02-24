use crate::events::events::Result;
use chrono::prelude::*;
use std::collections::hash_map::*;
use std::fs::File;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::*;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct Config {
    pub git: HashMap<String, crate::events::git::GitConfig>,
    pub email: HashMap<String, crate::events::email::EmailConfig>,
    pub ical: HashMap<String, crate::events::ical::IcalConfig>,
    pub redmine: HashMap<String, crate::events::redmine::RedmineConfig>,
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let config_folder = Self::config_folder()?;
        Ok(config_folder.join("config.toml"))
    }

    pub fn default_config() -> Config {
        Config {
            git: HashMap::new(),
            email: HashMap::new(),
            ical: HashMap::new(),
            redmine: HashMap::new(),
        }
    }

    pub fn read_config() -> Result<Config> {
        let config_file = Self::config_path()?;
        if !config_file.is_file() {
            return Ok(Self::default_config());
        }
        let mut contents = String::new();
        File::open(config_file)?.read_to_string(&mut contents)?;
        toml::from_str(&contents).map_err(|e| {
            // TODO verbose.. https://www.reddit.com/r/rust/comments/esueur/returning_trait_objects/
            Box::new(e) as Box<dyn error::Error>
        })
    }

    pub fn save_config(config: &Config) -> Result<()> {
        let mut file = File::create(Self::config_path()?)?;
        file.write_all(toml::to_string_pretty(config)?.as_bytes())?;
        Ok(())
    }

    #[cfg(unix)]
    fn set_private_folder(path: &PathBuf) -> Result<()> {
        let mut p = File::open(path)?.metadata()?.permissions();
        p.set_mode(0o700);
        fs::set_permissions(path, p)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_private_folder(_path: &PathBuf) -> Result<()> {
        Ok(())
    }

    pub fn config_folder() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().expect("Can't find your home folder?");
        let config_folder = home_dir.join(".cigale");
        if !config_folder.is_dir() {
            fs::create_dir(&config_folder)?;
            // we potentially put passwords in the config file...
            Self::set_private_folder(&config_folder)?;
        }
        Ok(config_folder)
    }

    /// cache handling

    pub fn get_cache_path(name: &str) -> Result<PathBuf> {
        let config_folder = Self::config_folder()?;
        Ok(config_folder.join(name))
    }

    pub fn get_cached_file(name: &str, date: &DateTime<Local>) -> Result<Option<String>> {
        let cache_file = Self::get_cache_path(name)?;
        if !cache_file.exists() {
            return Ok(None);
        }
        let metadata = std::fs::metadata(&cache_file)?;
        if DateTime::from(metadata.modified()?) >= *date {
            let mut contents = String::new();
            File::open(cache_file)?.read_to_string(&mut contents)?;
            Ok(Some(contents))
        } else {
            println!("{} cache too old, refetching", name);
            Ok(None)
        }
    }
}

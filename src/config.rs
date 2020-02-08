use std::collections::hash_map::*;
use std::io::Read;
use std::*;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct Config {
    pub git: HashMap<String, crate::events::git::Git>,
    pub email: HashMap<String, crate::events::email::Email>,
}

pub fn read_config() -> Result<Config, Box<dyn error::Error>> {
    let home_dir = dirs::home_dir().expect("Can't find your home folder?");
    let config_folder = home_dir.join(".cigale");
    if !config_folder.is_dir() {
        fs::create_dir(&config_folder)?;
    }
    let config_file = config_folder.join("config.toml");
    if !config_file.is_file() {
        return Ok(Config {
            git: HashMap::new(),
            email: HashMap::new(),
        });
    }
    let mut contents = String::new();
    std::fs::File::open(config_file)?.read_to_string(&mut contents)?;
    toml::from_str(&contents).map_err(|e| {
        // TODO verbose.. https://www.reddit.com/r/rust/comments/esueur/returning_trait_objects/
        Box::new(e) as Box<dyn error::Error>
    })
}

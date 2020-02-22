// not using the redmine Rest api because
// 1. unless the redmine admin greenlights it, a user may be unable to get an apikey
// 2. the redmine rest api doesn't offer an activity API https://www.redmine.org/issues/14872
//    without such an API, this would be very painful and very slow
use super::events::{ConfigType, Event, EventProvider, Result};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use std::collections::HashMap;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct RedmineConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

pub struct Redmine;
const SERVER_URL_KEY: &'static str = "Server URL";
const USERNAME_KEY: &'static str = "Username";
const PASSWORD_KEY: &'static str = "Password";

impl EventProvider for Redmine {
    fn name(&self) -> &'static str {
        "Redmine"
    }

    fn default_icon(&self) -> &'static str {
        "tasks"
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.redmine.keys().collect()
    }

    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (SERVER_URL_KEY, ConfigType::Text),
            (USERNAME_KEY, ConfigType::Text),
            (PASSWORD_KEY, ConfigType::Text), // TODO should be config type=password!
        ]
    }

    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String> {
        let mut h = HashMap::new();
        h.insert(
            SERVER_URL_KEY,
            config.redmine[config_name].server_url.to_string(),
        );
        h.insert(
            USERNAME_KEY,
            config.redmine[config_name].username.to_string(),
        );
        h.insert(
            PASSWORD_KEY,
            config.redmine[config_name].password.to_string(),
        );
        h
    }

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.redmine.insert(
            config_name,
            RedmineConfig {
                server_url: config_values.remove(SERVER_URL_KEY).unwrap(),
                username: config_values.remove(USERNAME_KEY).unwrap(),
                password: config_values.remove(PASSWORD_KEY).unwrap(),
            },
        );
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.redmine.remove(&config_name);
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: &Date<Local>,
    ) -> Result<Vec<Event>> {
        let redmine_config = &config.redmine[config_name];
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()
            .unwrap();

        let html = client
            .get(&format!("{}", redmine_config.server_url))
            .send()?
            .error_for_status()?
            .text()?;
        println!("{}", html);
        let doc = scraper::Html::parse_document(&html);
        let sel = scraper::Selector::parse("input[name=authenticity_token]").unwrap();
        let auth_token_node = doc.select(&sel).next().unwrap();
        let auth_token = auth_token_node.value().attr("value").unwrap();

        let res = client
            .post(&format!("{}/login", redmine_config.server_url))
            .form(&[
                ("username", &redmine_config.username),
                ("password", &redmine_config.password),
                ("login", &"Login".to_string()),
                ("utf8", &"✓".to_string()),
                ("back_url", &redmine_config.server_url),
                ("authenticity_token", &auth_token.to_string()),
            ])
            .send()?
            .error_for_status()?;
        println!("{:?}", res);

        let html = client
            .get(&format!("{}/activity", redmine_config.server_url))
            .send()?
            .error_for_status()?
            .text()?;
        println!("{}", html);
        Ok(vec![])
    }
}

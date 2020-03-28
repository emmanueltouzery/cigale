use super::email::Email;
use super::git::Git;
use super::gitlab::Gitlab;
use super::ical::Ical;
use super::redmine::Redmine;
use crate::config::Config;
use chrono::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::Instant;

#[derive(PartialEq, Copy, Clone)]
pub enum ConfigType {
    Text,
    Password,
    File,
    Folder,
    Combo,
}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait EventProvider: Sync {
    // TODO this could get derived automatically through a procedural macro
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)>;

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String>;

    // TODO this could get derived automatically through a procedural macro
    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String>;

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        config_values: HashMap<&'static str, String>,
    );

    fn field_values(
        &self,
        cur_values: &HashMap<&'static str, String>,
        field_name: &'static str,
    ) -> Result<Vec<String>>;

    fn remove_config(&self, config: &mut Config, config_name: String);

    fn name(&self) -> &'static str;

    fn default_icon(&self) -> &'static [u8];

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: Date<Local>,
    ) -> Result<Vec<Event>>;
}

pub fn get_event_providers() -> Vec<Box<dyn EventProvider>> {
    vec![
        Box::new(Git),
        Box::new(Email),
        Box::new(Ical),
        Box::new(Redmine),
        Box::new(Gitlab),
    ]
}

#[derive(Debug)]
struct ProviderError {
    pub provider_name: &'static str,
    pub config_name: String,
    pub err: Box<dyn Error>,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}: {}",
            self.provider_name, self.config_name, self.err
        )
    }
}

impl Error for ProviderError {}

/// lets us know from which event source the error came
impl ProviderError {
    fn new(provider_name: &'static str, config_name: String, err: Box<dyn Error>) -> ProviderError {
        ProviderError {
            provider_name,
            config_name,
            err,
        }
    }
}

pub fn get_all_events(config: Config, day: Date<Local>) -> Result<Vec<Event>> {
    let start = Instant::now();
    let mut events: Vec<Event> = get_event_providers()
        .iter()
        .flat_map(|ep| {
            ep.get_config_names(&config)
                .into_iter()
                .map(move |cfg_name| (ep, cfg_name))
        })
        .map(|(ep, cfg_name)| {
            let start_cfg = Instant::now();
            let result = ep.get_events(&config, cfg_name, day);
            println!(
                "Fetched events for {}/{} in {:?}",
                cfg_name,
                ep.name(),
                start_cfg.elapsed()
            );
            result
        })
        .collect::<Result<Vec<Vec<Event>>>>()?
        .into_iter()
        .flatten()
        .collect();
    events.sort_by_key(|e| e.event_time);
    println!("Fetched all events for {} in {:?}", day, start.elapsed());
    Ok(events)
}

#[derive(Clone, Debug, PartialEq)]
pub enum WordWrapMode {
    WordWrap,
    NoWordWrap,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventBody {
    PlainText(String),
    Markup(String, WordWrapMode),
}

impl EventBody {
    pub fn is_markup(&self) -> bool {
        match self {
            EventBody::Markup(_, _) => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EventBody::Markup(str, _) => &str,
            EventBody::PlainText(str) => &str,
        }
    }

    pub fn is_word_wrap(&self) -> bool {
        match self {
            EventBody::Markup(_, WordWrapMode::WordWrap) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub event_type_desc: &'static str,
    pub event_type_icon: &'static [u8],
    pub event_time: NaiveTime,
    pub event_info: String,
    pub event_contents_header: String,
    pub event_contents_body: EventBody,
    pub event_extra_details: Option<String>,
}

impl Event {
    pub fn new(
        event_type_desc: &'static str,
        event_type_icon: &'static [u8],
        event_time: NaiveTime,
        event_info: String,
        event_contents_header: String,
        event_contents_body: EventBody,
        event_extra_details: Option<String>,
    ) -> Event {
        Event {
            event_type_desc,
            event_type_icon,
            event_time,
            event_info,
            event_contents_header,
            event_contents_body,
            event_extra_details,
        }
    }
}

use super::email::Email;
use super::git::Git;
use super::ical::Ical;
use super::redmine::Redmine;
use crate::config::Config;
use chrono::prelude::*;
use std::collections::HashMap;
use std::error::Error;

#[derive(PartialEq, Copy, Clone)]
pub enum ConfigType {
    Text,
    Password,
    File,
    Folder,
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
    ]
}

fn get_events_for_event_provider(
    config: &Config,
    ep: &Box<dyn EventProvider>,
    day: Date<Local>,
) -> Result<Vec<Event>> {
    ep.get_config_names(&config)
        .iter()
        .map(|name| ep.get_events(&config, name, day))
        .collect::<Result<Vec<Vec<Event>>>>()
        .map(|es| es.into_iter().flatten().collect())
}

pub fn get_all_events(config: Config, day: Date<Local>) -> Result<Vec<Event>> {
    let mut events: Vec<Event> = get_event_providers()
        .iter()
        .map(|ep| get_events_for_event_provider(&config, ep, day))
        .collect::<Result<Vec<Vec<Event>>>>()?
        .into_iter()
        .flatten()
        .collect();
    events.sort_by_key(|e| e.event_time);
    Ok(events)
}

#[derive(Clone)]
pub enum EventBody {
    PlainText(String),
    Markup(String),
}

impl EventBody {
    pub fn is_markup(&self) -> bool {
        match self {
            EventBody::Markup(_) => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            EventBody::Markup(str) => &str,
            EventBody::PlainText(str) => &str,
        }
    }
}

#[derive(Clone)]
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

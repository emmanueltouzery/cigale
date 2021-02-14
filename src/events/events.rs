use super::email::Email;
use super::git::Git;
use super::gitlab::Gitlab;
use super::ical::Ical;
use super::redmine::Redmine;
use super::stackexchange::StackExchange;
use crate::config::Config;
use crate::icons::*;
use chrono::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::Instant;

#[derive(PartialEq, Copy, Clone)]
pub enum ConfigType {
    Text(&'static str),
    Password,
    File,
    Folder,
    Combo,
}

pub type Result<T> = std::result::Result<T, Box<dyn Error + Sync + Send>>;

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

    fn default_icon(&self) -> Icon;

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
        Box::new(StackExchange),
    ]
}

#[derive(Debug)]
struct ProviderError {
    pub provider_name: &'static str,
    pub config_name: String,
    pub err: Box<dyn Error + Send + Sync>,
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
    fn new(
        provider_name: &'static str,
        config_name: String,
        err: Box<dyn Error + Send + Sync>,
    ) -> ProviderError {
        ProviderError {
            provider_name,
            config_name,
            err,
        }
    }
}

pub fn get_all_events(config: Config, day: Date<Local>) -> Result<Vec<Event>> {
    let start = Instant::now();
    let eps = get_event_providers();
    let configs_to_fetch: Vec<(&Box<dyn EventProvider>, &String)> = eps
        .iter()
        .flat_map(|ep| {
            ep.get_config_names(&config)
                .into_iter()
                .map(move |cfg_name| (ep, cfg_name))
        })
        .collect();

    // use rayon's par_iter to fetch in parallel from multiple
    // event sources -- it's not CPU bound, but some sources
    // go to the network and parallelization helps a lot.
    // maybe I should force the size of the rayon's thread pool:
    // https://docs.rs/rayon/1.3.0/rayon/struct.ThreadPoolBuilder.html#method.build_global
    // because I think currently rayon will tie it to the number
    // of cores of the machine, but in our case it's really independent
    // as the tasks are IO-bound. Possibly I should enforce let's say
    // 3 threads always. But for now I'll leave the defaults.
    let mut events: Vec<Event> = configs_to_fetch
        .par_iter()
        .map(|(ep, cfg_name)| {
            let start_cfg = Instant::now();
            let result = ep.get_events(&config, cfg_name, day).map_err(|err| {
                Box::new(ProviderError::new(ep.name(), (*cfg_name).clone(), err))
                    as Box<dyn std::error::Error + Send + Sync>
            });
            log::info!(
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
    log::info!("Fetched all events for {} in {:?}", day, start.elapsed());
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
        matches!(self, EventBody::Markup(_, _))
    }

    pub fn as_str(&self) -> &str {
        match self {
            EventBody::Markup(str, _) => &str,
            EventBody::PlainText(str) => &str,
        }
    }

    pub fn is_word_wrap(&self) -> bool {
        matches!(self, EventBody::Markup(_, WordWrapMode::WordWrap))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub event_type_desc: &'static str,
    pub event_type_icon: Icon,
    pub event_time: NaiveTime,
    pub event_info: String,
    pub event_contents_header: String,
    pub event_contents_body: EventBody,
    pub event_extra_details: Option<String>,
}

impl Event {
    pub fn new(
        event_type_desc: &'static str,
        event_type_icon: Icon,
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

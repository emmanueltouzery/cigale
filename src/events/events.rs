use chrono::prelude::*;
use std::error::Error;

pub trait EventProvider: Sync {
    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>>;
}

// fn fold_events<T: EventProvider>(
//     acc: &mut Vec<Event>,
//     event_provider: (&String, &T),
// ) -> Result<Vec<Event>, Box<dyn Error>> {
//     acc.append(&mut ep.get_events(day)?);
//     Ok(acc)
// }

pub fn get_all_events(
    config: crate::config::Config,
    day: &Date<Local>,
) -> Result<Vec<Event>, Box<dyn Error>> {
    // TODO copy/paste. Tried with fold_events higher up, but failed so far.
    let mut events = config.git.iter().try_fold(
        Vec::new(),
        |mut acc, (ref _name, ref ep)| -> Result<Vec<Event>, Box<dyn Error>> {
            acc.append(&mut ep.get_events(day)?);
            Ok(acc)
        },
    )?;
    events.append(&mut config.email.iter().try_fold(
        Vec::new(),
        |mut acc, (ref _name, ref ep)| -> Result<Vec<Event>, Box<dyn Error>> {
            acc.append(&mut ep.get_events(day)?);
            Ok(acc)
        },
    )?);
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
    pub event_type_icon: &'static str,
    pub event_time: NaiveTime,
    pub event_info: String,
    pub event_contents_header: String,
    pub event_contents_body: EventBody,
    pub event_extra_details: Option<String>,
}

impl Event {
    pub fn new(
        event_type_desc: &'static str,
        event_type_icon: &'static str,
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

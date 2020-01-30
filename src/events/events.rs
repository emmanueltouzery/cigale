use chrono::prelude::*;

pub trait EventType {
    fn get_desc(&self) -> &'static str;
    fn get_icon(&self) -> &'static str;
    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>>;
}

pub fn get_all_events(day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    println!("get all events is called! {}", day);
    let git = super::git::Git {
        repoFolder: "/home/emmanuel/projects/bus/afc".to_string(),
    };
    let email = super::email::Email {
        mboxFilePath: "".to_string(),
    };

    let mut events = git.get_events(day)?;
    let mut emailEvents = email.get_events(day)?;
    events.append(&mut emailEvents);
    Ok(events)
}

#[derive(Clone)]
pub struct Event {
    pub event_type_desc: &'static str,
    pub event_type_icon: &'static str,
    pub event_time: String,
    pub event_info: String,
    pub event_contents: String,
    pub event_extra_details: Option<String>,
}

impl Event {
    pub fn new(
        event_type_desc: &'static str,
        event_type_icon: &'static str,
        event_time: String,
        event_info: String,
        event_contents: String,
        event_extra_details: Option<String>,
    ) -> Event {
        Event {
            event_type_desc,
            event_type_icon,
            event_time,
            event_info,
            event_contents,
            event_extra_details,
        }
    }
}

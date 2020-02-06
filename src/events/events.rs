use chrono::prelude::*;

pub trait EventProvider {
    fn get_desc(&self) -> &'static str;
    fn get_icon(&self) -> &'static str;
    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>>;
}

pub fn get_all_events(day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    let git = super::git::Git {
        repo_folder: "/home/emmanuel/projects/bus/afc".to_string(),
        commit_author: "Emmanuel Touzery".to_string(),
    };
    let email = super::email::Email {
        mbox_file_path: "/home/emmanuel/.thunderbird/sm8eskm1.default/Mail/mail.lecip-its.com/Sent"
            .to_string(),
    };

    let mut events = git.get_events(day)?;
    let mut email_events = email.get_events(day)?;
    events.append(&mut email_events);
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

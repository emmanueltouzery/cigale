pub trait EventTypeTrait {
    fn get_desc(&self) -> &str;
    fn get_icon(&self) -> &str;
}

#[derive(Clone, Copy)]
pub enum EventType {
    Git,
    Email,
}

impl EventTypeTrait for EventType {
    fn get_desc(&self) -> &str {
        match self {
            EventType::Git => "Git",
            EventType::Email => "Email",
        }
    }

    fn get_icon(&self) -> &str {
        match self {
            EventType::Git => "code-branch",
            EventType::Email => "envelope",
        }
    }
}

#[derive(Clone)]
pub struct Event {
    pub event_type: EventType,
    pub event_time: String,
    pub event_info: String,
    pub event_contents: String,
    pub event_extra_details: Option<String>,
}

impl Event {
    pub fn new(
        event_type: EventType,
        event_time: String,
        event_info: String,
        event_contents: String,
        event_extra_details: Option<String>,
    ) -> Event {
        Event {
            event_type,
            event_time,
            event_info,
            event_contents,
            event_extra_details,
        }
    }
}

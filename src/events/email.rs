use super::events::{Event, EventType};
use chrono::prelude::*;

pub struct Email {
    pub mboxFilePath: String, // Path
}

impl EventType for Email {
    fn get_desc(&self) -> &'static str {
        "Email"
    }

    fn get_icon(&self) -> &'static str {
        "envelope"
    }

    fn get_events(&self, day: &Date<Local>) -> Vec<Event> {
        vec![Event::new(
            self.get_desc(),
            self.get_icon(),
            "13:42".to_string(),
            format!("important email {}", day),
            "Hello John, Goodbye John".to_string(),
            Some("to: John Doe (john@example.com)".to_string()),
        )]
    }
}

use super::events::{Event, EventProvider};
use chrono::prelude::*;

pub struct Email {
    pub mboxFilePath: String, // Path
}

impl EventProvider for Email {
    fn get_desc(&self) -> &'static str {
        "Email"
    }

    fn get_icon(&self) -> &'static str {
        "envelope"
    }

    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(vec![Event::new(
            self.get_desc(),
            self.get_icon(),
            NaiveTime::from_hms(13, 42, 0),
            format!("important email {}", day),
            "Hello John, Goodbye John".to_string(),
            Some("to: John Doe (john@example.com)".to_string()),
        )])
    }
}

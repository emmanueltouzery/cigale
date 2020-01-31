use super::events::{Event, EventType};
use chrono::prelude::*;

// git2 revwalk
// https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs

pub struct Git {
    pub repoFolder: String, // Path
}

impl EventType for Git {
    fn get_desc(&self) -> &'static str {
        "Git"
    }

    fn get_icon(&self) -> &'static str {
        "code-branch"
    }

    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        if day < &Local.ymd(2020, 1, 1) {
            return Err(Box::<dyn std::error::Error>::from("too old"));
        }
        Ok(vec![Event::new(
            self.get_desc(),
            self.get_icon(),
            "12:56".to_string(),
            format!("Emmanuel Touzery, Jane Doe {}", day),
            "Commit message <b>details</b>".to_string(),
            Some("42 messages, lasted 2:30".to_string()),
        )])
    }
}

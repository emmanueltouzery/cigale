use super::events::{ConfigType, Event, EventBody, EventProvider, Result, WordWrapMode};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use itertools::{join, Itertools};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GitlabConfig {
    pub gitlab_url: String,
    pub personal_access_token: String,
}

pub struct Gitlab;
const GITLAB_URL_KEY: &str = "Gitlab URL";
const PERSONAL_TOKEN_KEY: &str = "Personal Access Token";

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabNote {
    body: String,
    #[serde(rename = "type")]
    note_type: String,
    noteable_type: String,
    position: Option<GitlabPosition>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabPosition {
    new_path: String,
    new_line: usize,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabEvent {
    action_name: String,
    target_type: Option<String>,
    target_title: Option<String>,
    created_at: DateTime<Local>,
    note: Option<GitlabNote>,
}

impl Gitlab {
    fn noteable_type_desc(note_type: &str) -> String {
        if note_type == "MergeRequest" {
            "Merge Request comment".to_string()
        } else {
            note_type.to_string()
        }
    }
}

impl EventProvider for Gitlab {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (GITLAB_URL_KEY, ConfigType::Text),
            (PERSONAL_TOKEN_KEY, ConfigType::Password),
        ]
    }

    fn name(&self) -> &'static str {
        "Gitlab"
    }

    fn default_icon(&self) -> &'static [u8] {
        crate::icons::FONTAWESOME_COMMENT_DOTS_SVG
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.gitlab.keys().collect()
    }

    fn field_values(
        &self,
        _cur_values: &HashMap<&'static str, String>,
        _field_name: &'static str,
    ) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String> {
        let mut h = HashMap::new();
        h.insert(
            GITLAB_URL_KEY,
            config.gitlab[config_name].gitlab_url.to_string(),
        );
        h.insert(
            PERSONAL_TOKEN_KEY,
            config.gitlab[config_name].personal_access_token.to_string(),
        );
        h
    }

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.gitlab.insert(
            config_name,
            GitlabConfig {
                gitlab_url: config_values.remove(GITLAB_URL_KEY).unwrap(),
                personal_access_token: config_values.remove(PERSONAL_TOKEN_KEY).unwrap(),
            },
        );
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.gitlab.remove(&config_name);
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: Date<Local>,
    ) -> Result<Vec<Event>> {
        let gitlab_config = &config.gitlab[config_name];
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()?;
        let url = format!(
            "{}/api/v4/events?after={}&before={}",
            &gitlab_config.gitlab_url,
            day.pred().format("%F"),
            day.succ().format("%F"),
        );
        let gitlab_events = client
            .get(&url)
            .header("PRIVATE-TOKEN", &gitlab_config.personal_access_token)
            .send()?
            .error_for_status()?
            .json::<Vec<GitlabEvent>>()?;

        let mut data_grouped: Vec<(&String, Vec<&GitlabEvent>)> = Vec::new();
        for (key, group) in &gitlab_events
            .iter()
            .filter(|evt| evt.note.is_some() && evt.target_title.is_some())
            .group_by(|evt| evt.target_title.as_ref().unwrap())
        {
            data_grouped.push((key, group.collect()));
        }
        let events = data_grouped
            .iter()
            .map(|(target_title, evts)| {
                let contents = join(
                    evts.iter().map(|evt| {
                        let note = evt.note.as_ref().unwrap();
                        format!(
                            "<b>{}</b>:{}\n    {}",
                            glib::markup_escape_text(
                                &note
                                    .position
                                    .as_ref()
                                    .map(|p| p.new_path.as_str())
                                    .unwrap_or("")
                            ),
                            note.position.as_ref().map(|p| p.new_line).unwrap_or(0),
                            glib::markup_escape_text(&note.body)
                        )
                    }),
                    "\n\n",
                );
                let note_type_desc = Self::noteable_type_desc(
                    &evts
                        .iter()
                        .next()
                        .unwrap()
                        .note
                        .as_ref()
                        .unwrap()
                        .noteable_type,
                );
                Event::new(
                    "Gitlab",
                    crate::icons::FONTAWESOME_COMMENT_DOTS_SVG,
                    evts.iter()
                        .min_by_key(|e| e.created_at)
                        .unwrap()
                        .created_at
                        .time(),
                    (*target_title).to_string(),
                    format!("{}: {}", note_type_desc, target_title),
                    EventBody::Markup(contents, WordWrapMode::WordWrap),
                    Some(note_type_desc),
                )
            })
            .collect();
        Ok(events)
    }
}

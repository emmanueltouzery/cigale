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
    note_type: Option<String>,
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
    target_iid: Option<usize>,
}

impl Gitlab {
    fn noteable_type_desc(note_type: &str) -> String {
        if note_type == "MergeRequest" {
            "Merge Request comment".to_string()
        } else {
            note_type.to_string()
        }
    }

    fn build_mr_comment_event(target_title: &String, evts: &Vec<&GitlabEvent>) -> Event {
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
    }

    fn gather_merge_request_comments(gitlab_events: &[GitlabEvent]) -> Vec<Event> {
        let mut data_grouped: Vec<(&String, Vec<&GitlabEvent>)> = Vec::new();
        for (key, group) in &gitlab_events
            .iter()
            .filter(|evt| evt.note.is_some() && evt.target_title.is_some())
            .group_by(|evt| evt.target_title.as_ref().unwrap())
        {
            data_grouped.push((key, group.collect()));
        }
        data_grouped
            .iter()
            .map(|(target_title, evts)| Self::build_mr_comment_event(target_title, evts))
            .collect()
    }

    fn gather_merge_request_accept_events(gitlab_events: &[GitlabEvent]) -> Vec<Event> {
        gitlab_events
            .iter()
            .filter(|evt| {
                evt.action_name == "accepted" && evt.target_type.as_deref() == Some("MergeRequest")
            })
            .map(|g_evt| {
                let body = format!(
                    "Merge Request #{} Accepted: {}",
                    g_evt.target_iid.unwrap(),
                    g_evt.target_title.as_ref().unwrap()
                );
                Event::new(
                    "Gitlab",
                    crate::icons::FONTAWESOME_CHECK_SQUARE_SVG,
                    g_evt.created_at.time(),
                    g_evt.target_title.as_ref().unwrap().to_string(),
                    body.clone(),
                    EventBody::PlainText(body),
                    Some("Merge Request accepted".to_string()),
                )
            })
            .collect()
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
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
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
        let gitlab_events: Vec<GitlabEvent> = client
            .get(&url)
            .header("PRIVATE-TOKEN", &gitlab_config.personal_access_token)
            .send()?
            .error_for_status()?
            .json::<Vec<GitlabEvent>>()?
            .into_iter()
            .filter(|e| e.created_at >= day_start && e.created_at < next_day_start)
            .collect();

        let mut events = Self::gather_merge_request_comments(&gitlab_events);
        events.append(&mut Self::gather_merge_request_accept_events(
            &gitlab_events,
        ));
        Ok(events)
    }
}

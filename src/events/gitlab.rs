use super::events::{ConfigType, Event, EventBody, EventProvider, Result, WordWrapMode};
use crate::config::Config;
use crate::icons::*;
use chrono::prelude::*;
use core::time::Duration;
use itertools::{join, Itertools};
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

type ProjectId = usize;

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
    noteable_iid: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabPosition {
    new_path: String,
    new_line: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabEvent {
    project_id: ProjectId,
    action_name: String,
    target_type: Option<String>,
    target_title: Option<String>,
    created_at: DateTime<Local>,
    note: Option<GitlabNote>,
    target_iid: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct GitlabProject {
    id: ProjectId,
    web_url: String,
}

impl Gitlab {
    fn noteable_type_desc(note_type: &str) -> String {
        if note_type == "MergeRequest" {
            "Merge Request".to_string()
        } else {
            note_type.to_string()
        }
    }

    fn build_openinbrowser_link(
        project_infos: &HashMap<ProjectId, String>,
        evt: &GitlabEvent,
    ) -> Cow<'static, str> {
        if let Some(iid) = evt.note.as_ref().unwrap().noteable_iid {
            Cow::from(format!(
                "<a href=\"{}{}{}\">Open in browser</a>\n\n",
                project_infos[&evt.project_id], "/merge_requests/", iid
            ))
        } else {
            Cow::from("")
        }
    }

    fn build_mr_comment_event(
        target_title: &str,
        evts: &[&GitlabEvent],
        project_infos: &HashMap<ProjectId, String>,
    ) -> Event {
        let contents = format!(
            "{}{}",
            Self::build_openinbrowser_link(project_infos, evts.first().unwrap()),
            join(
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
                        note.position.as_ref().and_then(|p| p.new_line).unwrap_or(0),
                        glib::markup_escape_text(&note.body)
                    )
                }),
                "\n\n",
            )
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
        let header = if let Some(iid) = evts
            .iter()
            .next()
            .unwrap()
            .note
            .as_ref()
            .unwrap()
            .noteable_iid
        {
            format!("{} #{}: {}", note_type_desc, iid, target_title)
        } else {
            format!("{}: {}", note_type_desc, target_title)
        };
        Event::new(
            "Gitlab",
            Icon::COMMENT_DOTS,
            evts.iter()
                .min_by_key(|e| e.created_at)
                .unwrap()
                .created_at
                .time(),
            (*target_title).to_string(),
            header,
            EventBody::Markup(contents, WordWrapMode::WordWrap),
            Some(note_type_desc),
        )
    }

    fn gather_merge_request_comments(
        gitlab_events: &[GitlabEvent],
        project_infos: &HashMap<ProjectId, String>,
    ) -> Vec<Event> {
        let mut data_grouped: Vec<(&String, Vec<&GitlabEvent>)> = Vec::new();
        for (key, group) in &gitlab_events
            .iter()
            .filter(|evt| {
                evt.note.is_some()
                    && evt.target_title.is_some()
                    && evt.target_type.as_deref() == Some("DiffNote")
            })
            .group_by(|evt| evt.target_title.as_ref().unwrap())
        {
            data_grouped.push((key, group.collect()));
        }
        data_grouped
            .iter()
            .map(|(target_title, evts)| {
                Self::build_mr_comment_event(target_title, evts, project_infos)
            })
            .collect()
    }

    fn gather_accept_events(
        gitlab_events: &[GitlabEvent],
        project_infos: &HashMap<ProjectId, String>,
    ) -> Vec<Event> {
        gitlab_events
            .iter()
            .filter(|evt| {
                (evt.action_name == "accepted"
                    && evt.target_type.as_deref() == Some("MergeRequest"))
                    || (evt.action_name == "closed" && evt.target_type.as_deref() == Some("Issue"))
            })
            .map(|g_evt| {
                let (desc, url_part) = match g_evt.target_type.as_deref().unwrap() {
                    "MergeRequest" => ("Merge Request", "/merge_requests/"),
                    "Issue" => ("Issue", "/issues/"),
                    x => {
                        log::error!("Unhandled target type: {:?}", g_evt.target_type);
                        (x, x)
                    }
                };
                let title = format!(
                    "{} #{} Accepted: {}",
                    desc,
                    g_evt.target_iid.unwrap(),
                    g_evt.target_title.as_ref().unwrap()
                );
                let body = format!(
                    "<a href=\"{}{}{}\">Open in browser</a>\n\n{}",
                    project_infos[&g_evt.project_id],
                    url_part,
                    g_evt.target_iid.unwrap(),
                    title
                );
                Event::new(
                    "Gitlab",
                    Icon::CHECK_SQUARE,
                    g_evt.created_at.time(),
                    g_evt.target_title.as_ref().unwrap().to_string(),
                    title,
                    EventBody::Markup(body, WordWrapMode::WordWrap),
                    Some(format!("{} accepted", desc)),
                )
            })
            .collect()
    }

    fn gather_issue_open_events(
        gitlab_events: &[GitlabEvent],
        project_infos: &HashMap<ProjectId, String>,
    ) -> Vec<Event> {
        gitlab_events
            .iter()
            .filter(|evt| {
                evt.action_name == "opened" && evt.target_type.as_deref() == Some("Issue")
            })
            .map(|g_evt| {
                let title = format!(
                    "Issue #{} Opened: {}",
                    g_evt.target_iid.unwrap(),
                    g_evt.target_title.as_ref().unwrap()
                );
                let body = format!(
                    "<a href=\"{}{}{}\">Open in browser</a>\n\n{}",
                    project_infos[&g_evt.project_id],
                    "/issues/",
                    g_evt.target_iid.unwrap(),
                    title
                );
                Event::new(
                    "Gitlab",
                    Icon::COMMENT_DOTS,
                    g_evt.created_at.time(),
                    g_evt.target_title.as_ref().unwrap().to_string(),
                    title,
                    EventBody::Markup(body, WordWrapMode::WordWrap),
                    Some("Issue opened".to_string()),
                )
            })
            .collect()
    }

    fn gather_issue_comment_events(
        gitlab_events: &[GitlabEvent],
        project_infos: &HashMap<ProjectId, String>,
    ) -> Vec<Event> {
        gitlab_events
            .iter()
            .filter(|evt| {
                evt.action_name == "commented on"
                    && evt.target_type.as_deref() == Some("Note")
                    && evt.note.is_some()
            })
            .map(|g_evt| {
                let title = if let Some(iid) = g_evt.note.as_ref().unwrap().noteable_iid {
                    format!(
                        "Issue #{} Comment: {}",
                        iid,
                        g_evt.target_title.as_ref().unwrap()
                    )
                } else {
                    format!("Comment: {}", g_evt.target_title.as_ref().unwrap())
                };
                let body = if let Some(iid) = g_evt.note.as_ref().unwrap().noteable_iid {
                    format!(
                        "<a href=\"{}{}{}\">Open in browser</a>\n\n{}",
                        project_infos[&g_evt.project_id], "/issues/", iid, title
                    )
                } else {
                    title.clone()
                };
                Event::new(
                    "Gitlab",
                    Icon::COMMENT_DOTS,
                    g_evt.created_at.time(),
                    g_evt.target_title.as_ref().unwrap().to_string(),
                    title,
                    EventBody::Markup(body, WordWrapMode::WordWrap),
                    Some("Issue comment".to_string()),
                )
            })
            .collect()
    }

    fn get_projects_info(
        config_name: &str,
        gitlab_config: &GitlabConfig,
        project_ids: &HashSet<ProjectId>,
    ) -> Result<HashMap<ProjectId, String>> {
        let cache = Config::get_cached_contents(
            &Gitlab,
            config_name,
            &Local.ymd(1970, 1, 1).and_hms(0, 0, 0),
        )?;
        match cache.and_then(|cached_json| {
            Self::get_projects_from_json_str(&cached_json, project_ids)
                .ok()
                .flatten()
        }) {
            Some(hash) => Ok(hash),
            None => {
                // either no cache or the cache doesn't know some of the
                // projects (it's outdated) => refetch & store to cache
                let response = Self::call_gitlab_rest(
                    "/api/v4/projects?simple=yes&membership=yes",
                    gitlab_config,
                )?;
                Config::write_to_cache(&Gitlab, config_name, &response)?;
                let hash = Self::get_projects_from_json_str(&response, project_ids)?
                    .ok_or("Can't find all projects?")?;
                Ok(hash)
            }
        }
    }

    fn get_projects_from_json_str(
        json_str: &str,
        project_ids: &HashSet<ProjectId>,
    ) -> Result<Option<HashMap<ProjectId, String>>> {
        let projects = serde_json::from_str::<Vec<GitlabProject>>(json_str)?;
        let filtered_projects: Vec<_> = projects
            .into_iter()
            .filter(|p| project_ids.contains(&p.id))
            .collect();
        if filtered_projects.len() == project_ids.len() {
            // found all the projects
            Ok(Some(
                filtered_projects
                    .into_iter()
                    .map(|p| (p.id, p.web_url))
                    .collect(),
            ))
        } else {
            // some projects are not known... Most likely we were reading from our
            // cache and a new project was added since the cache was populated,
            // and we'll need to update the cache.
            Ok(None)
        }
    }

    fn call_gitlab_rest(get_url: &str, gitlab_config: &GitlabConfig) -> Result<String> {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()?;
        let str = client
            .get(&format!("{}/{}", gitlab_config.gitlab_url, get_url))
            .header("PRIVATE-TOKEN", &gitlab_config.personal_access_token)
            .send()?
            .error_for_status()?
            .text()?;
        log::debug!("{}: {}", get_url, str);
        Ok(str)
    }
}

impl EventProvider for Gitlab {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (GITLAB_URL_KEY, ConfigType::Text("")),
            (PERSONAL_TOKEN_KEY, ConfigType::Password),
        ]
    }

    fn name(&self) -> &'static str {
        "Gitlab"
    }

    fn default_icon(&self) -> Icon {
        Icon::COMMENT_DOTS
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
        vec![
            (
                GITLAB_URL_KEY,
                config.gitlab[config_name].gitlab_url.to_string(),
            ),
            (
                PERSONAL_TOKEN_KEY,
                config.gitlab[config_name].personal_access_token.to_string(),
            ),
        ]
        .into_iter()
        .collect()
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
        let url = format!(
            "/api/v4/events?after={}&before={}",
            day.pred().format("%F"),
            day.succ().format("%F"),
        );

        let gitlab_events_str = Self::call_gitlab_rest(&url, &gitlab_config)?;

        let gitlab_events: Vec<_> = serde_json::from_str::<Vec<GitlabEvent>>(&gitlab_events_str)
            .map_err(|e| {
                format!(
                    "Failed parsing gitlab events {:?} -- {}",
                    e, gitlab_events_str
                )
            })?
            .into_iter()
            .filter(|e| e.created_at >= day_start && e.created_at < next_day_start)
            .collect();

        let project_infos = Self::get_projects_info(
            config_name,
            &gitlab_config,
            &gitlab_events.iter().map(|e| e.project_id).collect(),
        )?;
        log::debug!("project infos: {:?}", project_infos);

        let mut events = Self::gather_merge_request_comments(&gitlab_events, &project_infos);
        events.append(&mut Self::gather_accept_events(
            &gitlab_events,
            &project_infos,
        ));
        events.append(&mut Self::gather_issue_open_events(
            &gitlab_events,
            &project_infos,
        ));
        events.append(&mut Self::gather_issue_comment_events(
            &gitlab_events,
            &project_infos,
        ));
        Ok(events)
    }
}

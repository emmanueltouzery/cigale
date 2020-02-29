// not using the redmine Rest api because
// 1. unless the redmine admin greenlights it, a user may be unable to get an apikey
// 2. the redmine rest api doesn't offer an activity API https://www.redmine.org/issues/14872
//    without such an API, this would be very painful and very slow
use super::events::{ConfigType, Event, EventBody, EventProvider, Result, WordWrapMode};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const REDMINE_CACHE_FNAME: &str = "redmine-cache.html";

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct RedmineConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

pub struct Redmine;
const SERVER_URL_KEY: &str = "Server URL";
const USERNAME_KEY: &str = "Username";
const PASSWORD_KEY: &str = "Password";

// have a look at implementing Try here
// once try_trait stabilizes https://github.com/rust-lang/rust/issues/42327
// would allow to use ? a little more through this file
enum ActivityParseResult {
    Ok(Vec<Event>),
    Err(Box<dyn std::error::Error>),
    ReachedEndOfPage(Option<String>), // link to the previous page or None if no previous
}

impl Redmine {
    fn parse_date(date_str: &str) -> Result<Date<Local>> {
        if date_str == "Today" {
            Ok(Local::today())
        } else {
            let naive = NaiveDate::parse_from_str(date_str, "%m/%d/%Y")?;
            let local = Local
                .from_local_date(&naive)
                .single()
                .ok_or(format!("Can't convert {} to local time", naive))?;
            Ok(local)
        }
    }

    fn parse_time(time_str: &str) -> Result<NaiveTime> {
        Ok(NaiveTime::parse_from_str(&time_str, "%I:%M %p")?)
    }

    fn parse_events<'a>(
        redmine_config: &RedmineConfig,
        contents_elt: &scraper::element_ref::ElementRef<'a>,
    ) -> Result<Vec<Event>> {
        let description_sel = scraper::Selector::parse("span.description").unwrap();
        let link_sel = scraper::Selector::parse("dt.icon a").unwrap();
        let time_sel = scraper::Selector::parse("span.time").unwrap();
        let mut it_descriptions = contents_elt.select(&description_sel);
        let mut it_links = contents_elt.select(&link_sel);
        let mut it_times = contents_elt.select(&time_sel);
        let mut day_has_data = true;
        let mut result = vec![];
        while day_has_data {
            let next_time = it_times.next();
            day_has_data = next_time.is_some();
            if day_has_data {
                let time_elt = &next_time.unwrap();
                let time_str = time_elt.inner_html();
                let time = Self::parse_time(&time_str)?;
                let description_elt = &it_descriptions.next().unwrap();
                let link_elt = &it_links.next().unwrap();
                result.push(Event::new(
                    "Redmine",
                    crate::icons::FONTAWESOME_TASKS_SVG,
                    time,
                    link_elt.inner_html(),
                    link_elt.inner_html(),
                    EventBody::Markup(
                        format!(
                            "<a href=\"{}{}\">Open in the browser</a>\n{}",
                            redmine_config.server_url,
                            link_elt.value().attr("href").unwrap_or(""),
                            glib::markup_escape_text(&description_elt.inner_html()),
                        ),
                        WordWrapMode::WordWrap,
                    ),
                    None,
                ));
            }
        }
        Ok(result)
    }

    fn init_client(redmine_config: &RedmineConfig) -> Result<(reqwest::blocking::Client, String)> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()
            .unwrap();

        let html = client
            .get(&redmine_config.server_url)
            .send()?
            .error_for_status()?
            .text()?;
        let doc = scraper::Html::parse_document(&html);
        let sel = scraper::Selector::parse("input[name=authenticity_token]").unwrap();
        let auth_token_node = doc.select(&sel).next().unwrap();
        let auth_token = auth_token_node.value().attr("value").unwrap();

        let html = client
            .post(&format!("{}/login", redmine_config.server_url))
            .form(&[
                ("username", &redmine_config.username),
                ("password", &redmine_config.password),
                ("login", &"Login".to_string()),
                ("utf8", &"✓".to_string()),
                ("back_url", &redmine_config.server_url),
                ("authenticity_token", &auth_token.to_string()),
            ])
            .send()?
            .error_for_status()?
            .text()?;
        let doc = scraper::Html::parse_document(&html);
        let user_sel = scraper::Selector::parse("a.user.active").unwrap();
        let user_id = doc
            .select(&user_sel)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .replace("/users/", "");
        Ok((client, user_id))
    }

    fn fetch_activity_html(
        redmine_config: &RedmineConfig,
    ) -> Result<(reqwest::blocking::Client, String)> {
        let (client, user_id) = Self::init_client(redmine_config)?;

        let html = client
            .get(&format!(
                "{}/activity?user_id={}",
                redmine_config.server_url, user_id
            ))
            .send()?
            .error_for_status()?
            .text()?;
        let mut file = File::create(Config::get_cache_path(REDMINE_CACHE_FNAME)?)?;
        file.write_all(html.as_bytes())?;
        Ok((client, html))
    }

    fn parse_html(
        redmine_config: &RedmineConfig,
        day: Date<Local>,
        activity_html: &str,
    ) -> ActivityParseResult {
        let doc = scraper::Html::parse_document(&activity_html);
        let day_sel = scraper::Selector::parse("div#content div#activity h3").unwrap();
        let day_contents_sel =
            scraper::Selector::parse("div#content div#activity h3 + dl").unwrap();
        let mut it_day = doc.select(&day_sel);
        let mut it_contents = doc.select(&day_contents_sel);
        loop {
            let next_day = it_day.next();
            let contents = it_contents.next();
            match (next_day, contents) {
                (Some(day_elt), Some(contents_elt)) => {
                    // try_trait could maybe enable us to use the ? operator here
                    match Self::parse_date(&day_elt.inner_html()) {
                        Err(e) => return ActivityParseResult::Err(e),
                        Ok(cur_date) => {
                            if cur_date < day {
                                // passed the day, won't be any events this time.
                                return ActivityParseResult::Ok(vec![]);
                            }
                            if cur_date == day {
                                return match Self::parse_events(redmine_config, &contents_elt) {
                                    Err(e) => ActivityParseResult::Err(e),
                                    Ok(v) => ActivityParseResult::Ok(v),
                                };
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        // no matches in this page, search for the 'previous' paging link
        let previous_sel = scraper::Selector::parse("li.previous.page a").unwrap();
        let previous_url = doc
            .select(&previous_sel)
            .next()
            .and_then(|p| p.value().attr("href"));
        ActivityParseResult::ReachedEndOfPage(
            previous_url.map(|s| redmine_config.server_url.clone() + s),
        )
    }

    fn get_events_with_paging(
        day: Date<Local>,
        activity_html: String,
        redmine_config: &RedmineConfig,
        client_opt: Option<reqwest::blocking::Client>,
    ) -> Result<Vec<Event>> {
        match Self::parse_html(redmine_config, day, &activity_html) {
            ActivityParseResult::Ok(events) => Ok(events),
            ActivityParseResult::Err(e) => Err(e),
            ActivityParseResult::ReachedEndOfPage(None) => Ok(vec![]),
            ActivityParseResult::ReachedEndOfPage(Some(new_url)) => {
                // recursively check for the previous page
                let client = match client_opt {
                    Some(c) => c,
                    None => Self::init_client(redmine_config)?.0,
                };
                println!("Fetching {}", new_url);
                let html = client.get(&new_url).send()?.error_for_status()?.text()?;
                Self::get_events_with_paging(day, html, redmine_config, Some(client))
            }
        }
    }
}

impl EventProvider for Redmine {
    fn name(&self) -> &'static str {
        "Redmine"
    }

    fn default_icon(&self) -> &'static [u8] {
        crate::icons::FONTAWESOME_TASKS_SVG
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.redmine.keys().collect()
    }

    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (SERVER_URL_KEY, ConfigType::Text),
            (USERNAME_KEY, ConfigType::Text),
            (PASSWORD_KEY, ConfigType::Password),
        ]
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
            SERVER_URL_KEY,
            config.redmine[config_name].server_url.to_string(),
        );
        h.insert(
            USERNAME_KEY,
            config.redmine[config_name].username.to_string(),
        );
        h.insert(
            PASSWORD_KEY,
            config.redmine[config_name].password.to_string(),
        );
        h
    }

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.redmine.insert(
            config_name,
            RedmineConfig {
                server_url: config_values.remove(SERVER_URL_KEY).unwrap(),
                username: config_values.remove(USERNAME_KEY).unwrap(),
                password: config_values.remove(PASSWORD_KEY).unwrap(),
            },
        );
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.redmine.remove(&config_name);
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: Date<Local>,
    ) -> Result<Vec<Event>> {
        let redmine_config = &config.redmine[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let (client, activity_html) =
            match Config::get_cached_file(REDMINE_CACHE_FNAME, &next_day_start)? {
                Some(t) => Ok((None, t)),
                None => Self::fetch_activity_html(&redmine_config).map(|(a, b)| (Some(a), b)),
            }?;
        Self::get_events_with_paging(day, activity_html, redmine_config, client)
    }
}

// not using the redmine Rest api because
// 1. unless the redmine admin greenlights it, a user may be unable to get an apikey
// 2. the redmine rest api doesn't offer an activity API https://www.redmine.org/issues/14872
//    without such an API, this would be very painful and very slow
use super::events::{ConfigType, Event, EventBody, EventProvider, Result};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const REDMINE_CACHE_FNAME: &'static str = "redmine-cache.html";

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct RedmineConfig {
    pub server_url: String,
    pub username: String,
    pub password: String,
}

pub struct Redmine;
const SERVER_URL_KEY: &'static str = "Server URL";
const USERNAME_KEY: &'static str = "Username";
const PASSWORD_KEY: &'static str = "Password";

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

    fn parse_events<'a>(contents_elt: &scraper::element_ref::ElementRef<'a>) -> Result<Vec<Event>> {
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
                    "tasks",
                    time,
                    link_elt.inner_html(),
                    link_elt.inner_html(),
                    EventBody::PlainText(description_elt.inner_html()),
                    None,
                ));
            }
        }
        Ok(result)
    }

    fn fetch_activity_html(redmine_config: &RedmineConfig) -> Result<String> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()
            .unwrap();

        let html = client
            .get(&format!("{}", redmine_config.server_url))
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
        Ok(html)
    }
}

impl EventProvider for Redmine {
    fn name(&self) -> &'static str {
        "Redmine"
    }

    fn default_icon(&self) -> &'static str {
        "tasks"
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.redmine.keys().collect()
    }

    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (SERVER_URL_KEY, ConfigType::Text),
            (USERNAME_KEY, ConfigType::Text),
            (PASSWORD_KEY, ConfigType::Text), // TODO should be config type=password!
        ]
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
        day: &Date<Local>,
    ) -> Result<Vec<Event>> {
        let redmine_config = &config.redmine[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let activity_html = match Config::get_cached_file(REDMINE_CACHE_FNAME, &next_day_start)? {
            Some(t) => Ok(t),
            None => Self::fetch_activity_html(&redmine_config),
        }?;
        let doc = scraper::Html::parse_document(&activity_html);
        let day_sel = scraper::Selector::parse("div#content div#activity h3").unwrap();
        let day_contents_sel =
            scraper::Selector::parse("div#content div#activity h3 + dl").unwrap();
        let mut it_day = doc.select(&day_sel);
        let mut it_contents = doc.select(&day_contents_sel);
        let mut page_has_data = true;
        while page_has_data {
            let next_day = it_day.next();
            let contents = it_contents.next();
            page_has_data = next_day.is_some();
            if page_has_data {
                let day_elt = &next_day.unwrap();
                let cur_date = Self::parse_date(&day_elt.inner_html())?;
                if cur_date < *day {
                    // passed the day, won't be any events this time.
                    return Ok(vec![]);
                }
                if cur_date == *day {
                    let contents_elt = &contents.unwrap();
                    return Self::parse_events(&contents_elt);
                }
            }
        }
        Ok(vec![])
    }
}

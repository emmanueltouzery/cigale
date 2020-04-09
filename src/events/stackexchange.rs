// as far I know the official stackexchange API does not expose user votes
// https://stackapps.com/questions/4725/list-of-votes-by-authenticated-users
// https://meta.stackexchange.com/questions/288217/how-could-i-get-my-own-vote-activity-from-api
// so I have to scrap the website
// my understand is that scraping is acceptable if there is no alternative:
// https://meta.stackexchange.com/a/446/218504
use super::events::{ConfigType, Event, EventBody, EventProvider, Result, WordWrapMode};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use std::collections::HashMap;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct StackExchangeConfig {
    pub exchange_site_url: String,
    pub username: String,
    pub password: String,
}

pub struct StackExchange;
const EXCHANGE_SITE_URL: &str = "Stack Exchange site url";
const USERNAME_KEY: &str = "username";
const PASSWORD_KEY: &str = "password";

impl StackExchange {
    fn url_escape(msg: &str) -> String {
        msg.replace("/", "%2f").replace(":", "%3a")
    }

    fn login(
        client: &reqwest::blocking::Client,
        stackexchange_config: &StackExchangeConfig,
    ) -> Result<String> {
        let home_body = Self::html_get(
            client,
            stackexchange_config,
            &format!(
                "/users/login?ssrc=head&returnurl={}",
                Self::url_escape(&stackexchange_config.exchange_site_url)
            ),
        )?;
        let doc = scraper::Html::parse_document(&home_body);
        let sel_fkey = scraper::Selector::parse("input[name=fkey]").unwrap();
        let fkey_node = doc
            .select(&sel_fkey)
            .next()
            .ok_or("login: can't find fkey?")?;
        let fkey = fkey_node
            .value()
            .attr("value")
            .ok_or("login: can't find fkey value?")?;

        let resp = client
            .post(&format!(
                "{}/users/login?ssrc=head&returnurl={}",
                stackexchange_config.exchange_site_url,
                Self::url_escape(&stackexchange_config.exchange_site_url)
            ))
            .form(&[
                ("ssrc", "head"),
                ("fkey", fkey),
                ("email", &stackexchange_config.username),
                ("password", &stackexchange_config.password),
                ("oauth_version", ""),
                ("oauth_server", ""),
            ])
            .send()?
            .error_for_status()?;
        let html = resp.text()?;
        if html.contains("Human verification") && html.contains("Are you a human being?") {
            Err("Login rejected: human verification failed".into())
        } else {
            Ok(html)
        }
    }

    fn html_get(
        client: &reqwest::blocking::Client,
        stackexchange_config: &StackExchangeConfig,
        url_path: &str,
    ) -> Result<String> {
        log::debug!(
            "getting {}",
            &format!("{}{}", stackexchange_config.exchange_site_url, url_path)
        );
        let resp = client
            .get(&format!(
                "{}{}",
                stackexchange_config.exchange_site_url, url_path
            ))
            .send()?
            .error_for_status()?;

        let html = resp.text()?;
        log::debug!(
            "{}{}: got back html {}",
            stackexchange_config.exchange_site_url,
            url_path,
            html
        );
        Ok(html)
    }

    fn get_user_page_url(html: &str) -> Result<String> {
        let doc = scraper::Html::parse_document(html);
        let sel_userpage = scraper::Selector::parse("a.my-profile.js-gps-track").unwrap();
        let userpage_node = doc
            .select(&sel_userpage)
            .next()
            .ok_or("Can't find the user page link")?;
        userpage_node
            .value()
            .attr("href")
            .map(|s| s.to_string())
            .ok_or_else(|| "Can't find the link to the user page".into())
    }

    fn get_votes_page_html(
        config_name: &str,
        stackexchange_config: &StackExchangeConfig,
    ) -> Result<String> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .user_agent(format!(
                "Cigale/{} (https://github.com/emmanueltouzery/cigale)",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .connection_verbose(true)
            .build()?;

        let html = Self::login(&client, &stackexchange_config)?;

        let userpage_link = Self::get_user_page_url(&html)?;

        let votes_page_html = Self::html_get(
            &client,
            stackexchange_config,
            &format!("{}?tab=votes", userpage_link),
        )?;

        Config::write_to_cache(&StackExchange, config_name, &votes_page_html)?;
        Ok(votes_page_html)
    }

    fn get_votes(
        votes_page_html: &str,
        stackexchange_config: &StackExchangeConfig,
        day_start: DateTime<Local>,
        next_day_start: DateTime<Local>,
    ) -> Result<Vec<Event>> {
        let doc = scraper::Html::parse_document(votes_page_html);
        let sel_vote_urls = scraper::Selector::parse(
            "table.history-table a.answer-hyperlink,table.history-table a.question-hyperlink",
        )
        .unwrap();
        let sel_vote_dates = scraper::Selector::parse("table.history-table div.date").unwrap();
        let mut vote_urls = doc.select(&sel_vote_urls);
        let sel_date_brick = scraper::Selector::parse("div.date_brick").unwrap();
        Ok(doc
            .select(&sel_vote_dates)
            .filter_map(|date_node| {
                let title_link = vote_urls
                    .next()
                    .map(|n| (n.inner_html(), n.value().attr("href")));
                let date_str = date_node.value().attr("title").or_else(|| {
                    date_node
                        .select(&sel_date_brick)
                        .next()
                        .and_then(|n| n.value().attr("title"))
                });
                let date: Option<DateTime<Local>> = date_str
                    .and_then(|d_str| {
                        DateTime::parse_from_str(&d_str.replace("Z", "+00"), "%Y-%m-%d %H:%M:%S%#z")
                            .ok()
                    })
                    .map(DateTime::from);
                log::debug!("{:?} - {:?}", date_str, date);
                if let (Some(date), Some((title, Some(link)))) = (date, title_link) {
                    Some((date, title, link))
                } else {
                    None
                }
            })
            .filter(|(date, _, _)| date >= &day_start && date < &next_day_start)
            .map(|(date, title, link)| {
                Event::new(
                    "S.Exch",
                    crate::icons::FONTAWESOME_THUMBS_UP_SVG,
                    date.time(),
                    title.clone(),
                    format!("Vote: {}", title),
                    EventBody::Markup(
                        format!(
                            "<a href=\"{}{}\">Open in the browser</a>\n\nStack Exchange vote: {}",
                            stackexchange_config.exchange_site_url,
                            link,
                            stackexchange_config.exchange_site_url
                        ),
                        WordWrapMode::WordWrap,
                    ),
                    Some("Vote".to_string()),
                )
            })
            .collect())
    }
}

impl EventProvider for StackExchange {
    fn name(&self) -> &'static str {
        "StackExchange"
    }

    fn default_icon(&self) -> &'static [u8] {
        crate::icons::FONTAWESOME_THUMBS_UP_SVG
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.stackexchange.keys().collect()
    }

    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (
                EXCHANGE_SITE_URL,
                ConfigType::Text("https://stackoverflow.com"),
            ),
            (USERNAME_KEY, ConfigType::Text("")),
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
        vec![
            (
                EXCHANGE_SITE_URL,
                config.stackexchange[config_name]
                    .exchange_site_url
                    .to_string(),
            ),
            (
                USERNAME_KEY,
                config.stackexchange[config_name].username.to_string(),
            ),
            (
                PASSWORD_KEY,
                config.stackexchange[config_name].password.to_string(),
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
        config.stackexchange.insert(
            config_name,
            StackExchangeConfig {
                exchange_site_url: config_values.remove(EXCHANGE_SITE_URL).unwrap(),
                username: config_values.remove(USERNAME_KEY).unwrap(),
                password: config_values.remove(PASSWORD_KEY).unwrap(),
            },
        );
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.stackexchange.remove(&config_name);
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: Date<Local>,
    ) -> Result<Vec<Event>> {
        log::debug!("stackexchange::get_events");
        let stackexchange_config = &config.stackexchange[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);

        let votes_page_html =
            match Config::get_cached_contents(&StackExchange, config_name, &next_day_start)? {
                Some(t) => Ok(t),
                None => Self::get_votes_page_html(config_name, &stackexchange_config),
            }?;

        Self::get_votes(
            &votes_page_html,
            &stackexchange_config,
            day_start,
            next_day_start,
        )
    }
}

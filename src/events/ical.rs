use super::events::{ConfigType, Event, EventBody, EventProvider, Result};
use crate::config::Config;
use chrono::prelude::*;
use core::time::Duration;
use ical::parser::ical::component::IcalEvent;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const ICAL_CACHE_FNAME: &'static str = "ical-cache.ical";

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct IcalConfig {
    pub ical_url: String,
}

impl Ical {
    fn get_property_value<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a str> {
        event
            .properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|s| s.value.as_ref().map(|s| s.as_str()))
    }

    fn get_property_value_any<'a>(event: &'a IcalEvent, names: &[&str]) -> Option<&'a str> {
        names
            .iter()
            .find(|n| Ical::get_property_value(event, n).is_some())
            .and_then(|n| Ical::get_property_value(event, n))
    }

    fn parse_ical_date(ical_date_str: &str) -> Option<DateTime<Local>> {
        Utc.datetime_from_str(ical_date_str, "%Y%m%dT%H%M%SZ")
            .ok()
            .map(DateTime::from)
            .or_else(|| Local.datetime_from_str(ical_date_str, "%Y%m%dT%H%M%S").ok())
            .or_else(|| {
                // pure laziness from me here. that chrono function wants a time component,
                // i give it a time component.
                // Otherwise not the same as earlier: we assume local time not UTC here.
                Local
                    .datetime_from_str(format!("{}T00:00:00", ical_date_str).as_str(), "%Y%m%dT%T")
                    .ok()
            })
    }

    fn fetch_ical(ical_url: &str) -> Result<String> {
        let r = reqwest::blocking::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .build()?
            .get(ical_url)
            .send()?
            .error_for_status()?
            .text()?;
        let mut file = File::create(Config::get_cache_path(ICAL_CACHE_FNAME)?)?;
        file.write_all(r.as_bytes())?;
        Ok(r)
    }

    fn add_event_if_in_range(
        event: &IcalEvent,
        day_start: &DateTime<Local>,
        next_day_start: &DateTime<Local>,
        result: &mut Vec<Event>,
    ) {
        let start = Ical::get_property_value(&event, "DTSTART");
        let end = Ical::get_property_value(&event, "DTEND");
        let summary =
            Ical::get_property_value_any(&event, &vec!["SUMMARY", "DESCRIPTION", "LOCATION"]);
        match (
            start.and_then(Ical::parse_ical_date),
            end.and_then(Ical::parse_ical_date),
            summary,
        ) {
            (Some(st), end_dt, Some(summ)) => {
                if st >= *day_start && st < *next_day_start {
                    result.push(Ical::build_event(summ, st, end_dt));
                }
            }
            _ => println!("Skipping event without start or summary: {:?}", event),
        }
    }

    fn build_event(summ: &str, st: DateTime<Local>, end_dt: Option<DateTime<Local>>) -> Event {
        let summary = summ.replace("\\,", ",");
        let extra_info = end_dt.map(|e| {
            let duration = e - st;
            format!(
                "End: {}; duration: {}:{:02}",
                e.format("%H:%M"),
                duration.num_hours(),
                duration.num_minutes()
            )
        });
        Event::new(
            "Ical",
            "calendar-alt",
            st.time(),
            summary.to_string(),
            summary,
            EventBody::PlainText(
                extra_info
                    .as_ref()
                    .map(|i| i.clone())
                    .unwrap_or_else(|| "".to_string()),
            ),
            extra_info,
        )
    }
}

const URL_KEY: &str = "Ical URL";

pub struct Ical;

impl EventProvider for Ical {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![(URL_KEY, ConfigType::Text)]
    }

    fn name(&self) -> &'static str {
        "Ical"
    }

    fn default_icon(&self) -> &'static str {
        "calendar-alt"
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.ical.keys().collect()
    }

    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String> {
        let mut h = HashMap::new();
        h.insert(URL_KEY, config.ical[config_name].ical_url.to_string());
        h
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.ical.remove(&config_name);
    }

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.ical.insert(
            config_name,
            IcalConfig {
                ical_url: config_values.remove(URL_KEY).unwrap(),
            },
        );
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: &Date<Local>,
    ) -> Result<Vec<Event>> {
        let ical_config = &config.ical[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let ical_text = match Config::get_cached_file(ICAL_CACHE_FNAME, &next_day_start)? {
            Some(t) => Ok(t),
            None => Ical::fetch_ical(&ical_config.ical_url),
        }?;
        let bytes = ical_text.as_bytes();
        let reader = ical::IcalParser::new(std::io::BufReader::new(bytes));
        let mut result = vec![];
        for line in reader {
            // the ical library's error type doesn't implement std::error::Error conversion
            // so it complicates using the '?' operator in our case
            match line {
                Ok(l) => {
                    for event in l.events {
                        Ical::add_event_if_in_range(
                            &event,
                            &day_start,
                            &next_day_start,
                            &mut result,
                        );
                    }
                }
                Err(_) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Ical error",
                    )))
                }
            }
        }
        Ok(result)
    }
}

#[test]
fn it_parses_ical_dates_correctly() {
    assert_eq!(
        FixedOffset::east(3600).ymd(2020, 2, 9).and_hms(15, 30, 50),
        Ical::parse_ical_date(&"20200209T143050Z".to_string()).unwrap()
    );
    // in practice entries with time which don't contain the timezone inline
    // have a separate ical entry: Property{name=DTSTART, params: {TZID: .., value: ..}}
    // but for now i'll just assume local time.
    assert_eq!(
        FixedOffset::east(3600).ymd(2020, 2, 9).and_hms(14, 30, 50),
        Ical::parse_ical_date(&"20200209T143050".to_string()).unwrap()
    );
    assert_eq!(
        FixedOffset::east(7200).ymd(2014, 3, 31).and_hms(0, 0, 0),
        Ical::parse_ical_date(&"20140331".to_string()).unwrap()
    );
}

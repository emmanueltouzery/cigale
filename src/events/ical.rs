use super::events::{Event, EventBody, EventProvider};
use chrono::prelude::*;
use ical::parser::ical::component::IcalEvent;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::result::Result;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct Ical {
    pub ical_url: String,
}

impl Ical {
    fn get_property_value<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a String> {
        event
            .properties
            .iter()
            .find(|p| p.name == name)
            .and_then(|s| s.value.as_ref())
    }

    fn get_property_value_any<'a>(event: &'a IcalEvent, names: &Vec<&str>) -> Option<&'a String> {
        names
            .iter()
            .find(|n| Ical::get_property_value(event, n).is_some())
            .and_then(|n| Ical::get_property_value(event, n))
    }

    fn parse_ical_date(ical_date_str: &String) -> Option<DateTime<Local>> {
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

    fn get_cache_path() -> Result<PathBuf, Box<dyn Error>> {
        let config_folder = crate::config::config_folder()?;
        Ok(config_folder.join("ical-cache.ical"))
    }

    fn get_cached_ical(date: &DateTime<Local>) -> Result<Option<String>, Box<dyn Error>> {
        let cache_file = Ical::get_cache_path()?;
        if !cache_file.exists() {
            return Ok(None);
        }
        let metadata = std::fs::metadata(&cache_file)?;
        if DateTime::from(metadata.modified()?) >= *date {
            let mut contents = String::new();
            File::open(cache_file)?.read_to_string(&mut contents)?;
            Ok(Some(contents))
        } else {
            println!("ical cache too old, refetching");
            Ok(None)
        }
    }

    fn fetch_ical(ical_url: &String) -> Result<String, Box<dyn Error>> {
        let r = reqwest::blocking::get(ical_url)?.text()?;
        let mut file = File::create(Ical::get_cache_path()?)?;
        file.write_all(r.as_bytes())?;
        Ok(r)
    }
}

impl EventProvider for Ical {
    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn Error>> {
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let ical_text = match Ical::get_cached_ical(&next_day_start)? {
            Some(t) => Ok(t),
            None => Ical::fetch_ical(&self.ical_url),
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
                        // for event in line.unwrap().events {
                        let start = Ical::get_property_value(&event, "DTSTART");
                        let summary = Ical::get_property_value_any(
                            &event,
                            &vec!["SUMMARY", "DESCRIPTION", "LOCATION"],
                        );
                        match (start.and_then(Ical::parse_ical_date), summary) {
                            (Some(st), Some(summ)) => {
                                if st >= day_start && st < next_day_start {
                                    let summary = summ.replace("\\,", ",");
                                    result.push(Event::new(
                                        "Ical",
                                        "calendar-alt",
                                        st.time(),
                                        summary.to_string(),
                                        summary.to_string(),
                                        EventBody::PlainText("".to_string()),
                                        None,
                                    ))
                                }
                            }
                            _ => println!("Skipping event without start or summary: {:?}", event),
                        }
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

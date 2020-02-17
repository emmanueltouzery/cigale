use super::events::{ConfigType, Event, EventBody, EventProvider, Result};
use crate::config::Config;
use chrono::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::*;

const BUF_SIZE: u64 = 4096;

// let mut separator_bytes = "\nFrom ".to_string().into_bytes();
// separator_bytes.reverse();
// could use lazy_static! but a dependency for that...
const SEPARATOR_BYTES: [u8; 6] = [
    ' ' as u8, 'm' as u8, 'o' as u8, 'r' as u8, 'F' as u8, '\n' as u8,
];

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct EmailConfig {
    pub mbox_file_path: String, // Path
}

struct ParsingState<'a> {
    bytes_left: u64,
    reader: &'a mut BufReader<File>,
}

impl Email {
    // re-reading the buffer from the file for each new email, but i rely on the bufreader too
    fn read_next_mail(
        buf: &mut Vec<u8>,
        parsing_state: &mut ParsingState,
    ) -> Result<Option<Vec<u8>>> {
        let mut email_contents: Vec<u8> = vec![];
        let mut separator_idx = 0;

        loop {
            if parsing_state.bytes_left == 0 {
                return Ok(None);
            }
            let cur_buf = Email::read_into_buffer(buf, parsing_state)?;

            for i in 0..cur_buf.len() {
                let cur = cur_buf[i];
                let byte_matches = cur == SEPARATOR_BYTES[separator_idx];
                let mut matches = false;
                if byte_matches && separator_idx == SEPARATOR_BYTES.len() - 1 {
                    // matching in the middle of the file.
                    // not interested in the extra \n so I take only [0..i]
                    matches = true;
                    email_contents.extend(cur_buf[0..i].iter());
                } else if separator_idx == SEPARATOR_BYTES.len() - 2
                    && parsing_state.bytes_left as usize - (i + 1) == 0
                {
                    // hit the beginning of the file (bytes_left - (i+1) == 0)
                    // => we don't require the leading \n from the separator bytes
                    // do collect the current letter too [0..(i+1)]
                    matches = true;
                    email_contents.extend(cur_buf[0..(i + 1)].iter());
                }
                if matches {
                    // found the marker for the beginning of the email
                    email_contents.reverse();
                    parsing_state.bytes_left -= (i + 1) as u64;
                    parsing_state
                        .reader
                        .seek(SeekFrom::Start(parsing_state.bytes_left))?;
                    return Ok(Some(email_contents));
                }
                if byte_matches {
                    separator_idx += 1;
                } else {
                    separator_idx = 0;
                }
            }
            email_contents.extend(cur_buf.iter());
            parsing_state.bytes_left -= cur_buf.len() as u64;
        }
    }

    fn read_into_buffer<'a>(
        buf: &'a mut Vec<u8>,
        parsing_state: &mut ParsingState,
    ) -> Result<&'a [u8]> {
        let cur_buf = if parsing_state.bytes_left as usize > buf.len() {
            &mut buf[0..] // can fill in the whole buffer
        } else {
            &mut buf[0..parsing_state.bytes_left as usize] // less than BUF_SIZE left to read
        };
        parsing_state
            .reader
            .seek(SeekFrom::Current(-(cur_buf.len() as i64)))?;
        parsing_state.reader.read_exact(cur_buf)?;
        // reading moved us back after the buffer => get back where we were
        parsing_state
            .reader
            .seek(SeekFrom::Current(-(cur_buf.len() as i64)))?;
        cur_buf.reverse(); // we'll read from end to beginning
        Ok(cur_buf)
    }

    fn get_header_val(headers: &Vec<mailparse::MailHeader>, header_name: &str) -> Option<String> {
        headers
            .iter()
            // TODO change to Result::contains when it stabilizes
            .find(|h| h.get_key().ok() == Some(header_name.to_string()))
            .and_then(|h| h.get_value().ok())
    }

    fn parse_email_headers_date(headers: &Vec<mailparse::MailHeader>) -> Option<DateTime<Local>> {
        Email::get_header_val(headers, "Date").and_then(|d_str| Email::parse_email_date(&d_str))
    }

    // some date strings end with " (CET)" timezone specifiers, but rust
    // format strings can't parse that:
    // %Z _Formatting only_: Local time zone name.
    // often we don't need them, so drop them.
    // this function is dumb, will dump the final 6 bytes if the
    // string is long enough. don't want to add a regex lib
    // dependency, don't feel like doing it more precisely.
    fn drop_string_tz_if_present(dt_str: &str) -> &str {
        if dt_str.len() > 6 {
            &dt_str[..(dt_str.len() - 6)]
        } else {
            dt_str
        }
    }

    fn parse_email_date(dt_str: &str) -> Option<DateTime<Local>> {
        DateTime::parse_from_rfc2822(&dt_str)
            .ok()
            .or_else(|| {
                DateTime::parse_from_str(
                    Email::drop_string_tz_if_present(dt_str),
                    "%a, %-d %b %Y %T %z",
                )
                .ok()
            })
            .map(|d| DateTime::from(d))
            .or_else(|| Local.datetime_from_str(dt_str, "%b %d %T %Y").ok())
            .or_else(|| Local.datetime_from_str(dt_str, "%a %b %e %T %Y").ok())
    }

    // skip emails which are newer than the date i'm interested in.
    // remember we're reading from the end.
    // it's ok to just read headers for now (I just want the date)
    fn find_first_mail_sent_before(
        buf: &mut Vec<u8>,
        parsing_state: &mut ParsingState,
        next_day_start: &DateTime<Local>,
    ) -> Result<Option<(Vec<u8>, DateTime<Local>)>> {
        loop {
            let email_bytes = Email::read_next_mail(buf, parsing_state)?;
            let email_headers = email_bytes
                .as_ref()
                .map(|bytes| mailparse::parse_headers(bytes))
                .transpose()?;
            let email_date = email_headers.and_then(|h| Email::parse_email_headers_date(&h.0));
            match email_date {
                None => {
                    return Ok(None); // no more emails
                }
                // the DateTime::from is to convert the TZ
                Some(date) if date < DateTime::from(*next_day_start) => {
                    // first date before my end date
                    return Ok(Some((email_bytes.unwrap(), date)));
                }
                Some(_) => {} // email, but after my end date
            }
        }
    }

    fn email_to_event(
        email_contents: &mailparse::ParsedMail,
        email_date: &DateTime<Local>,
    ) -> Result<Event> {
        let message = if email_contents.subparts.len() > 1 {
            email_contents.subparts[0].get_body()? // TODO check the mimetype, i want text, not html
        } else {
            email_contents.get_body()?
        };
        let email_subject =
            Email::get_header_val(&email_contents.headers, "Subject").unwrap_or("-".to_string());
        Ok(Event::new(
            "Email",
            "envelope",
            email_date.time(),
            email_subject.clone(),
            email_subject,
            EventBody::PlainText(message),
            Email::get_header_val(&email_contents.headers, "To"),
        ))
    }

    fn read_emails_until_day_start(
        buf: &mut Vec<u8>,
        day_start: &DateTime<Local>,
        parsing_state: &mut ParsingState,
    ) -> Result<Vec<Event>> {
        // now read the emails i'm interested in.
        // i'll read one-too-many email bodies (and I'll read
        // a header for the second time right now) but no biggie
        let mut result = vec![];
        loop {
            // the nest match doesn't look too great to my haskeller's eyes,
            // but i tried to carry the value through options,
            // as is done in find_first_mail_sent_before(), and it looked worse.
            match Email::read_next_mail(buf, parsing_state)? {
                None => return Ok(result),
                Some(email_bytes) => {
                    let email_contents = mailparse::parse_mail(&email_bytes)?;
                    let email_date = Email::parse_email_headers_date(&email_contents.headers);
                    match email_date.filter(|d| d >= day_start) {
                        None => return Ok(result),
                        Some(d) => result.push(Email::email_to_event(&email_contents, &d)?),
                    }
                }
            }
        }
    }
}

pub struct Email;

const MBOX_FILE_PATH_KEY: &'static str = "Mbox file path";

impl EventProvider for Email {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![(MBOX_FILE_PATH_KEY, ConfigType::Path)]
    }

    fn name(&self) -> &'static str {
        "Email"
    }

    fn default_icon(&self) -> &'static str {
        "envelope"
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.email.keys().collect()
    }

    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String> {
        let mut h = HashMap::new();
        h.insert(
            MBOX_FILE_PATH_KEY,
            config.email[config_name].mbox_file_path.to_string(),
        );
        h
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.email.remove(&config_name);
    }

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.email.insert(
            config_name,
            EmailConfig {
                mbox_file_path: config_values.remove(MBOX_FILE_PATH_KEY).unwrap(),
            },
        );
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: &Date<Local>,
    ) -> Result<Vec<Event>> {
        let email_config = &config.email[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let mut buf = vec![0; BUF_SIZE as usize];
        let file = File::open(&email_config.mbox_file_path)?;
        // i "double buffer". probably OK.
        let mut reader = BufReader::new(file);
        let cur_pos_end = reader.seek(SeekFrom::End(0))?;
        let mut parsing_state = ParsingState {
            reader: &mut reader,
            bytes_left: cur_pos_end,
        };
        // we go from the end. so we first search for an email sent
        // _before_ the end date we're interested in.
        let first_mail =
            Email::find_first_mail_sent_before(&mut buf, &mut parsing_state, &next_day_start)?;
        if let Some((email_bytes, email_date)) = first_mail {
            if email_date < day_start {
                // no emails match
                return Ok(vec![]);
            }
            let email_contents = mailparse::parse_mail(&email_bytes)?;
            // read until the first email sent before
            // the start date we're interested in.
            let mut emails =
                Email::read_emails_until_day_start(&mut buf, &day_start, &mut parsing_state)?;
            // add the first email now (append is faster than prepend, and sorting is done later)
            emails.push(Email::email_to_event(&email_contents, &email_date)?);
            Ok(emails)
        } else {
            // no emails match
            Ok(vec![])
        }
    }
}

#[test]
fn it_can_extract_two_short_emails() {
    let mut buf = vec![0; BUF_SIZE as usize];
    let file = File::open("tests/two_short_emails.txt").unwrap();
    let mut reader = BufReader::new(file);
    let cur_pos_end = reader.seek(SeekFrom::End(0)).unwrap();
    let mut parsing_state = ParsingState {
        reader: &mut reader,
        bytes_left: cur_pos_end,
    };

    let email = Email::read_next_mail(&mut buf, &mut parsing_state)
        .unwrap()
        .unwrap();
    assert_eq!("From b\nbye a\n", String::from_utf8(email).unwrap());
    assert_eq!(11, parsing_state.bytes_left);

    let email2 = Email::read_next_mail(&mut buf, &mut parsing_state)
        .unwrap()
        .unwrap();
    assert_eq!("From a\nhi b", String::from_utf8(email2).unwrap());

    let email3 = Email::read_next_mail(&mut buf, &mut parsing_state).unwrap();
    assert_eq!(true, email3.is_none());
}

#[test]
fn it_parses_multiple_email_date_formats() {
    let expected = FixedOffset::east(7200).ymd(2013, 9, 27).and_hms(20, 46, 35);
    assert_eq!(
        expected,
        Email::parse_email_date("Sep 27 20:46:35 2013").unwrap()
    );
    assert_eq!(
        expected,
        Email::parse_email_date("Fri, 27 Sep 2013 20:46:35 +0200").unwrap()
    );
    let expected2 = FixedOffset::east(3600).ymd(2014, 11, 3).and_hms(7, 54, 9);
    assert_eq!(
        expected2,
        Email::parse_email_date("Mon Nov  3 07:54:09 2014").unwrap() // notice the extra space
    );
    let expected2 = FixedOffset::east(3600).ymd(2014, 11, 3).and_hms(7, 54, 9);
    assert_eq!(
        expected2,
        Email::parse_email_date("Mon, 3 Nov 2014 07:54:09 +0100 (CET)").unwrap()
    );
    assert_eq!(
        expected2,
        Email::parse_email_date("Mon, 3 Nov 2014 07:54:09 +0100").unwrap()
    );
    assert_eq!(
        expected,
        Email::parse_email_date("Fri, 27 Sep 2013 18:46:35 GMT").unwrap()
    );
}

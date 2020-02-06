use super::events::{Event, EventBody, EventProvider};
use chrono::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::*;
use std::result::Result;

const BUF_SIZE: u64 = 4096;

// let mut separator_bytes = "\nFrom ".to_string().into_bytes();
// separator_bytes.reverse();
// could use lazy_static! but a dependency for that...
const SEPARATOR_BYTES: [u8; 6] = [
    ' ' as u8, 'm' as u8, 'o' as u8, 'r' as u8, 'F' as u8, '\n' as u8,
];

pub struct Email {
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
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut email_contents: Vec<u8> = vec![];
        let mut separator_idx = 0;

        loop {
            let cur_buf = if parsing_state.bytes_left as usize > buf.len() {
                &mut buf[0..]
            } else {
                &mut buf[0..parsing_state.bytes_left as usize]
            };
            parsing_state
                .reader
                .seek(SeekFrom::Current(-(cur_buf.len() as i64)))?;
            parsing_state.reader.read_exact(cur_buf)?;
            // reading moved us back after the buffer => get back where we were
            parsing_state
                .reader
                .seek(SeekFrom::Current(-(cur_buf.len() as i64)))?;
            cur_buf.reverse();

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
                    email_contents.reverse();
                    parsing_state.bytes_left -= (i + 1) as u64;
                    parsing_state
                        .reader
                        .seek(SeekFrom::Start(parsing_state.bytes_left))?;
                    return Ok(email_contents);
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

    fn parse_email_date(dt_str: &str) -> Option<DateTime<Local>> {
        DateTime::parse_from_rfc2822(&dt_str)
            .ok()
            .map(|d| DateTime::from(d))
            .or_else(|| Local.datetime_from_str(dt_str, "%b %d %T %Y").ok())
    }
}

impl EventProvider for Email {
    fn get_desc(&self) -> &'static str {
        "Email"
    }

    fn get_icon(&self) -> &'static str {
        "envelope"
    }

    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let mut buf = vec![0; BUF_SIZE as usize];
        let file = File::open(&self.mbox_file_path)?;
        // i "double buffer". probably OK.
        let mut reader = BufReader::new(file);
        let cur_pos_end = reader.seek(SeekFrom::End(0))?;
        let mut parsing_state = ParsingState {
            reader: &mut reader,
            bytes_left: cur_pos_end,
        };
        let mut email_bytes;
        let mut email_date;

        // first skip emails which are newer than the date i'm interested in.
        // it's ok to just read headers for now (I just want the date)
        loop {
            email_bytes = Email::read_next_mail(&mut buf, &mut parsing_state)?; // TODO stop reading when i'm out of emails

            let (email_headers, _) = mailparse::parse_headers(&email_bytes)?;
            email_date = Email::parse_email_headers_date(&email_headers);

            // TODO i should skip if i can't parse the date, or something
            if email_date
                .filter(|d| d < &DateTime::from(next_day_start)) // the from is to convert the TZ
                .is_some()
            {
                // first date before my end date
                break;
            }
        }

        // now read the emails i'm interested in.
        // i'll read one-too-many email bodies (and I'll read
        // a header for the second time right now) but no biggie
        let mut result = vec![];
        loop {
            let email_contents = mailparse::parse_mail(&email_bytes)?;
            let email_date = Email::parse_email_headers_date(&email_contents.headers);
            if email_date.is_none()
                || email_date
                    .filter(|d| d < &DateTime::from(day_start)) // the from is to convert the TZ
                    .is_some()
            {
                // first date before my start date
                break;
            }
            let message = if email_contents.subparts.len() > 1 {
                email_contents.subparts[0].get_body()? // TODO check the mimetype, i want text, not html
            } else {
                email_contents.get_body()?
            };
            result.push(Event::new(
                self.get_desc(),
                self.get_icon(),
                email_date.unwrap().time(),
                Email::get_header_val(&email_contents.headers, "Subject")
                    .unwrap_or("-".to_string()),
                "??".to_string(), // TODO
                EventBody::PlainText(message),
                Email::get_header_val(&email_contents.headers, "To"),
            ));
            email_bytes = Email::read_next_mail(&mut buf, &mut parsing_state)?; // TODO stop reading when i'm out of emails
        }

        Ok(result)
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

    let email = Email::read_next_mail(&mut buf, &mut parsing_state).unwrap();
    assert_eq!("From b\nbye a\n", String::from_utf8(email).unwrap());
    assert_eq!(11, parsing_state.bytes_left);

    let email2 = Email::read_next_mail(&mut buf, &mut parsing_state).unwrap();
    assert_eq!("From a\nhi b", String::from_utf8(email2).unwrap());
}

#[test]
fn it_parses_multiple_email_date_formats() {
    // TODO complete these tests, plus this doesn't pass
    let expected = DateTime::<Local>::from(Utc.ymd(2013, 9, 27).and_hms(19, 46, 35));
    assert_eq!(
        expected,
        Email::parse_email_date("Sep 27 20:46:35 2013").unwrap()
    );
    assert_eq!(
        expected,
        Email::parse_email_date("Fri, 27 Sep 2013 20:46:35 +0100").unwrap()
    );
}
// assertEqual "test zoned" expected (parseEmailDate "")
// assertEqual "test zoned" expected (parseEmailDate "Fri Sep 27 20:46:35 2013")
// assertEqual "test extra space" expected1 (parseEmailDate "Mon Nov  3 07:54:09 2014")
// assertEqual "test another" expected2 (parseEmailDate "Tue, 9 Dec 2014 06:27:27 +0100 (CET)")
// assertEqual "yet another" expected3 (parseEmailDate "Wed, 1 Jul 2015 08:22:43 +0200")
// assertEqual "really??" expected4 (parseEmailDate "Wed, 11 Nov 2015 14:00:51 GMT")
//     where
//         expected = LocalTime (fromGregorian 2013 9 27) (TimeOfDay 20 46 35)
//         expected1 = LocalTime (fromGregorian 2014 11 3) (TimeOfDay 7 54 9)
//         expected2 = LocalTime (fromGregorian 2014 12 9) (TimeOfDay 6 27 27)
//         expected3 = LocalTime (fromGregorian 2015 07 1) (TimeOfDay 8 22 43)
//         expected4 = LocalTime (fromGregorian 2015 11 11) (TimeOfDay 14 0 51)

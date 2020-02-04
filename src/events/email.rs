use super::events::{Event, EventProvider};
use chrono::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::*;
use std::result::Result;

const BUF_SIZE: u64 = 4096;

// let mut separator_bytes = "\nFrom: ".to_string().into_bytes();
// separator_bytes.reverse();
// could use lazy_static! but a dependency for that...
const SEPARATOR_BYTES: [u8; 7] = [
    ' ' as u8, ':' as u8, 'm' as u8, 'o' as u8, 'r' as u8, 'F' as u8, '\n' as u8,
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
        }
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
        let mut buf = vec![0; BUF_SIZE as usize];
        let file = File::open(&self.mbox_file_path)?;
        // i "double buffer". probably OK.
        let mut reader = BufReader::new(file);
        let cur_pos_end = reader.seek(SeekFrom::End(0))?;
        let mut parsing_state = ParsingState {
            reader: &mut reader,
            bytes_left: cur_pos_end,
        };

        let email_bytes = Email::read_next_mail(&mut buf, &mut parsing_state)?;
        let email_contents = mailparse::parse_mail(&email_bytes)?;
        println!("{}", email_contents.headers[0].get_key()?);
        println!("{}", email_contents.headers[0].get_value()?);
        println!("{}", email_contents.get_body()?);
        for subpart in &email_contents.subparts {
            println!("{}", subpart.get_body()?);
        }

        Ok(vec![Event::new(
            self.get_desc(),
            self.get_icon(),
            NaiveTime::from_hms(13, 42, 0),
            format!("important email {}", day),
            "Hello John, Goodbye John".to_string(),
            "".to_string(),
            Some("to: John Doe (john@example.com)".to_string()),
        )])
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
    assert_eq!("From: b\nbye a\n", String::from_utf8(email).unwrap());
    assert_eq!(12, parsing_state.bytes_left);

    let email2 = Email::read_next_mail(&mut buf, &mut parsing_state).unwrap();
    assert_eq!("From: a\nhi b", String::from_utf8(email2).unwrap());
}

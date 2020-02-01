use super::events::{Event, EventProvider};
use chrono::prelude::*;
use git2::{Commit, Repository};

// git2 revwalk
// https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs

pub struct Git {
    pub repo_folder: String, // Path
    pub commit_author: String,
}

impl Git {
    fn git2_time_to_datetime(time: git2::Time) -> DateTime<Local> {
        Utc.timestamp(time.seconds(), 0).with_timezone(&Local)
    }
}

impl EventProvider for Git {
    fn get_desc(&self) -> &'static str {
        "Git"
    }

    fn get_icon(&self) -> &'static str {
        "code-branch"
    }

    fn get_events(&self, day: &Date<Local>) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let repo = Repository::open(&self.repo_folder)?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(/*git2::Sort::REVERSE |*/ git2::Sort::TIME);
        revwalk.push_head()?;
        let mut commits: Vec<Commit> = revwalk
            .map(|r| {
                let oid = r?;
                repo.find_commit(oid)
            })
            .filter_map(|c| match c {
                Ok(commit) => Some(commit),
                Err(e) => {
                    println!("Error walking the revisions {}, skipping", e);
                    None
                }
            })
            .take_while(|c| {
                let commit_date = Git::git2_time_to_datetime(c.time());
                commit_date >= day_start
            })
            .filter(|c| {
                let commit_date = Git::git2_time_to_datetime(c.time());
                // TODO move to option.contains when it stabilizes https://github.com/rust-lang/rust/issues/62358
                commit_date < next_day_start && c.author().name() == Some(&self.commit_author)
            })
            .collect();
        commits.reverse();
        Ok(commits
            .iter()
            .map(|c| {
                let commit_date = Git::git2_time_to_datetime(c.time());
                Event::new(
                    self.get_desc(),
                    self.get_icon(),
                    commit_date.time(),
                    c.summary().unwrap_or("").to_string(),
                    c.message().unwrap_or("").to_string(),
                    Some("42 messages, lasted 2:30".to_string()),
                )
            })
            .collect())
    }
}

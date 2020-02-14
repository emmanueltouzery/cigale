use super::events::{ConfigType, Event, EventBody, EventProvider, Result};
use crate::config::Config;
use chrono::prelude::*;
use git2::{Commit, Repository};
use std::collections::HashMap;

// git2 revwalk
// https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug)]
pub struct GitConfig {
    pub repo_folder: String, // Path
    pub commit_author: String,
}

impl Git {
    fn git2_time_to_datetime(time: git2::Time) -> DateTime<Local> {
        Utc.timestamp(time.seconds(), 0).with_timezone(&Local)
    }

    fn get_commit_diff<'a>(repo: &'a Repository, c: &Commit) -> Option<git2::Diff<'a>> {
        if c.parent_count() > 1 {
            return None;
        }
        let commit_tree = c.tree().ok()?;
        let parent = c.parent(0).ok()?;
        let parent_tree = parent.tree().ok()?;
        repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
            .ok()
    }

    fn get_commit_full_diffstr(diff: &git2::Diff) -> Option<String> {
        let stats = diff.stats().ok()?;
        stats
            .to_buf(git2::DiffStatsFormat::FULL, 1024)
            .ok()
            .and_then(|b| b.as_str().map(|s| s.to_string()))
    }

    fn get_commit_extra_info<'a>(diff: &git2::Diff<'a>) -> Option<String> {
        // not done here. i want to get the list of files and copy the
        // getcommitExtraInfo algo from the cigale haskell version.
        let mut files_touched = vec![];
        let mut file_cb = |diff_delta: git2::DiffDelta<'_>, _count| {
            if let Some(path) = diff_delta.new_file().path() {
                files_touched.push(path.to_owned());
            }
            if let Some(path) = diff_delta.old_file().path() {
                files_touched.push(path.to_owned());
            }
            true
        };
        diff.foreach(&mut file_cb, None, None, None).ok()?;
        Some(Git::get_files_root(&files_touched))
    }

    // common prefix to all the files
    fn get_files_root(files: &Vec<std::path::PathBuf>) -> String {
        let paths_for_each_file: Vec<Vec<&str>> = files
            .iter()
            .filter_map(|f| f.iter().map(|c| c.to_str()).collect())
            .collect();
        let shortest_path = paths_for_each_file
            .iter()
            .map(|chars| chars.len())
            .min()
            .unwrap_or(0);
        let mut common_prefix = vec![];
        for idx in 0..shortest_path {
            let first_component = paths_for_each_file[0][idx];
            if !paths_for_each_file
                .iter()
                .all(|chars| chars[idx] == first_component)
            {
                break;
            }
            common_prefix.push(first_component);
        }
        common_prefix.join("/")
    }
}

pub struct Git;
const REPO_FOLDER_KEY: &'static str = "Repository folder";
const COMMIT_AUTHOR_KEY: &'static str = "Commit Author";

impl EventProvider for Git {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (REPO_FOLDER_KEY, ConfigType::Path),
            (COMMIT_AUTHOR_KEY, ConfigType::Text),
        ]
    }

    fn name(&self) -> &'static str {
        "Git"
    }

    fn default_icon(&self) -> &'static str {
        "code-branch"
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.git.keys().collect()
    }

    fn get_config_values(
        &self,
        config: &Config,
        config_name: &str,
    ) -> HashMap<&'static str, String> {
        let mut h = HashMap::new();
        h.insert(
            REPO_FOLDER_KEY,
            config.git[config_name].repo_folder.to_string(),
        );
        h.insert(
            COMMIT_AUTHOR_KEY,
            config.git[config_name].commit_author.to_string(),
        );
        h
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: &Date<Local>,
    ) -> Result<Vec<Event>> {
        let git_config = &config.git[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let repo = Repository::open(&git_config.repo_folder)?;
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
                commit_date < next_day_start && c.author().name() == Some(&git_config.commit_author)
            })
            .collect();
        commits.reverse();
        Ok(commits
            .iter()
            .map(|c| {
                let commit_date = Git::git2_time_to_datetime(c.time());
                let diff = Git::get_commit_diff(&repo, &c);
                let contents_header = c.message().unwrap_or("").to_string();
                let (contents, extra_details) = match diff {
                    None => ("".to_string(), None),
                    Some(d) => (
                        "<span font-family=\"monospace\">".to_owned()
                            + &Git::get_commit_full_diffstr(&d).unwrap_or("".to_string())
                            + "</span>",
                        Git::get_commit_extra_info(&d),
                    ),
                };
                Event::new(
                    "Git",
                    "code-branch",
                    commit_date.time(),
                    c.summary().unwrap_or("").to_string(),
                    contents_header,
                    EventBody::Markup(contents),
                    extra_details,
                )
            })
            .collect())
    }
}

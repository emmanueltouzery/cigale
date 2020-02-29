use super::events::{ConfigType, Event, EventBody, EventProvider, Result, WordWrapMode};
use crate::config::Config;
use chrono::prelude::*;
use git2::{Commit, Repository};
use std::collections::{HashMap, HashSet};

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
    fn get_files_root(files: &[std::path::PathBuf]) -> String {
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
const REPO_FOLDER_KEY: &str = "Repository folder";
const COMMIT_AUTHOR_KEY: &str = "Commit Author";

impl EventProvider for Git {
    fn get_config_fields(&self) -> Vec<(&'static str, ConfigType)> {
        vec![
            (REPO_FOLDER_KEY, ConfigType::Folder),
            (COMMIT_AUTHOR_KEY, ConfigType::Combo),
        ]
    }

    fn name(&self) -> &'static str {
        "Git"
    }

    fn default_icon(&self) -> &'static [u8] {
        crate::icons::FONTAWESOME_CODE_BRANCH_SVG
    }

    fn get_config_names<'a>(&self, config: &'a Config) -> Vec<&'a String> {
        config.git.keys().collect()
    }

    fn field_values(
        &self,
        cur_values: &HashMap<&'static str, String>,
        field_name: &'static str,
    ) -> Result<Vec<String>> {
        // for the 'commit author' combo box, we offer the list
        // of authors for the repo. This is quite slow though,
        // hopefully there is a faster way?
        // https://stackoverflow.com/questions/60464449/get-the-list-of-authors-in-a-git-repository-efficiently-with-libgit2
        let git_path = cur_values
            .get(REPO_FOLDER_KEY)
            .map(|s| s.as_str())
            .unwrap_or_else(|| "");
        if field_name != COMMIT_AUTHOR_KEY || git_path.is_empty() {
            return Ok(Vec::new());
        }
        let repo = Repository::open(&git_path)?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        let mut authors: Vec<String> = revwalk
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
            .fold(HashSet::new(), |mut sofar, cur| {
                if let Some(name) = cur.author().name() {
                    sofar.insert(name.to_string());
                }
                sofar
            })
            .into_iter()
            .collect();
        authors.sort();
        Ok(authors)
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

    fn add_config_values(
        &self,
        config: &mut Config,
        config_name: String,
        mut config_values: HashMap<&'static str, String>,
    ) {
        config.git.insert(
            config_name,
            GitConfig {
                repo_folder: config_values.remove(REPO_FOLDER_KEY).unwrap(),
                commit_author: config_values.remove(COMMIT_AUTHOR_KEY).unwrap(),
            },
        );
    }

    fn remove_config(&self, config: &mut Config, config_name: String) {
        config.git.remove(&config_name);
    }

    fn get_events(
        &self,
        config: &Config,
        config_name: &str,
        day: Date<Local>,
    ) -> Result<Vec<Event>> {
        let git_config = &config.git[config_name];
        let day_start = day.and_hms(0, 0, 0);
        let next_day_start = day_start + chrono::Duration::days(1);
        let repo = Repository::open(&git_config.repo_folder)?;
        let mut all_commits = HashMap::new();
        for branch in repo
            .branches(Some(git2::BranchType::Local))?
            .filter_map(|b| b.ok())
        {
            if let Some(branch_oid) = branch.0.get().target() {
                let branch_name = branch.0.name().ok().flatten().map(|s| s.to_string());
                let branch_head = repo.find_commit(branch_oid)?;
                let branch_head_date = Git::git2_time_to_datetime(branch_head.time());
                if branch_head_date < day_start {
                    // early abort: quite a lot faster than starting a useless revwalk
                    continue;
                }
                let mut revwalk = repo.revwalk()?;
                revwalk.set_sorting(/*git2::Sort::REVERSE |*/ git2::Sort::TIME);
                revwalk.push(branch_oid)?;
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
                        commit_date < next_day_start
                            && c.author().name() == Some(&git_config.commit_author)
                    })
                    .collect();
                commits.reverse();
                all_commits.insert(branch_name.unwrap_or_else(|| "".to_string()), commits);
            }
        }
        let mut result = all_commits
            .iter()
            .flat_map(|(branch, commits)| {
                let rrepo = &repo;
                commits.iter().map(move |c| {
                    let branch = branch.clone();
                    let commit_date = Git::git2_time_to_datetime(c.time());
                    let diff = Git::get_commit_diff(rrepo, &c);
                    let contents_header = c.message().unwrap_or("").to_string();
                    let (contents, extra_details) = match diff {
                        None => (branch.clone(), None),
                        Some(d) => (
                            ("<span font-family=\"monospace\">".to_owned()
                                + &branch
                                + "\n\n"
                                + &Git::get_commit_full_diffstr(&d)
                                    .unwrap_or_else(|| "".to_string())
                                + "</span>"),
                            Git::get_commit_extra_info(&d),
                        ),
                    };
                    Event::new(
                        "Git",
                        crate::icons::FONTAWESOME_CODE_BRANCH_SVG,
                        commit_date.time(),
                        c.summary().unwrap_or("").to_string(),
                        contents_header,
                        EventBody::Markup(contents, WordWrapMode::NoWordWrap),
                        extra_details,
                    )
                })
            })
            .collect::<Vec<Event>>();
        result.sort_by_key(|e| e.event_time); // need to sort for the dedup to work
        result.dedup_by(|e1, e2| {
            // deduplicate identical commits seen in different branches
            // (the body will be different since we put the branch name there)
            e1.event_time == e2.event_time
                && e1.event_contents_header == e2.event_contents_header
                && e1.event_info == e2.event_info
        });
        Ok(result)
    }
}

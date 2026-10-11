use crate::models::CommitInfo;
use anyhow::Result;
use chrono::DateTime;
use git2::{Repository, Sort};
use std::path::Path;

/// One page of commits reachable from HEAD.
#[derive(Clone, Debug)]
pub struct CommitPage {
    pub commits: Vec<CommitInfo>,
    pub has_more: bool,
}

pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
    Ok(Repository::open(path)?)
}

pub fn is_valid_repo<P: AsRef<Path>>(path: P) -> bool {
    Repository::open(path).is_ok()
}

/// Loads up to `limit` commits starting at `offset`, newest first.
pub fn load_commits(repo: &Repository, limit: usize, offset: usize) -> Result<CommitPage> {
    // an empty repository has no HEAD commit yet
    if repo.head().is_err() {
        return Ok(CommitPage {
            commits: Vec::new(),
            has_more: false,
        });
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
    revwalk.push_head()?;

    let mut commits = Vec::new();
    let mut has_more = false;

    for oid in revwalk.skip(offset) {
        if commits.len() == limit {
            has_more = true;
            break;
        }

        let commit = repo.find_commit(oid?)?;
        let hash = commit.id().to_string();
        let message = String::from_utf8_lossy(commit.message_bytes()).into_owned();
        let date = DateTime::from_timestamp(commit.time().seconds(), 0)
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M")
            .to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();

        commits.push(CommitInfo::new(hash, message, date, author));
    }

    Ok(CommitPage { commits, has_more })
}

/// Name of the checked-out branch, or `None` for detached HEAD or an unborn branch.
pub fn current_branch(repo: &Repository) -> Option<String> {
    if repo.head_detached().unwrap_or(false) {
        return None;
    }
    repo.head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(str::to_string))
}

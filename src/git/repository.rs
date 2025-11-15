use crate::models::CommitInfo;
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::Repository;
use std::path::Path;

pub trait GitRepository {
    fn load_commits(&self, limit: usize, offset: usize) -> Result<Vec<CommitInfo>>;
    fn get_current_branch(&self) -> Option<String>;
}

pub struct GitRepositoryImpl {
    repo: Repository,
}

impl GitRepositoryImpl {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn is_valid_repo<P: AsRef<Path>>(path: P) -> bool {
        Repository::open(path).is_ok()
    }
}

impl GitRepository for GitRepositoryImpl {
    fn load_commits(&self, limit: usize, offset: usize) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        let end_index = offset + limit;

        for (i, oid) in revwalk.enumerate() {
            if i >= end_index {
                break;
            }
            if i < offset {
                continue;
            }

            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            let message = commit
                .message()
                .unwrap_or("(no message)")
                .lines()
                .next()
                .unwrap_or("(no message)")
                .to_string();

            let time = commit.time();
            let datetime = DateTime::<Utc>::from_timestamp(time.seconds(), 0).unwrap_or_default();
            let date_str = datetime.format("%Y-%m-%d %H:%M").to_string();

            let author = commit.author().name().unwrap_or("Unknown").to_string();

            let hash_str = oid.to_string();
            let short_hash = hash_str.chars().take(8).collect::<String>();

            commits.push(CommitInfo::new(
                hash_str, short_hash, message, date_str, author,
            ));
        }

        Ok(commits)
    }

    fn get_current_branch(&self) -> Option<String> {
        match self.repo.head() {
            Ok(head) => head.shorthand().map(|s| s.to_string()),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_repo() {
        let current_dir = std::env::current_dir().unwrap();
        let _ = GitRepositoryImpl::is_valid_repo(current_dir);
    }
}

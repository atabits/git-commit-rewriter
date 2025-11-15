use crate::git::commands::{
    get_git_log, get_original_refs, restore_original_refs, run_git_filter_branch,
};
use crate::git::repository::{GitRepository, GitRepositoryImpl};
use crate::models::PreviewData;
use anyhow::Result;
use git2::Repository;
use std::path::Path;

pub fn rewrite_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: &str,
    new_message: &str,
    modify_all_branches: bool,
    branch_name: Option<&str>,
) -> Result<PreviewData> {
    let repo = Repository::open(repo_path.as_ref())?;
    let target_oid = git2::Oid::from_str(commit_hash)?;
    let target_commit = repo.find_commit(target_oid)?;

    let old_message = target_commit
        .message()
        .unwrap_or("(no message)")
        .lines()
        .next()
        .unwrap_or("(no message)")
        .to_string();

    let mut child = run_git_filter_branch(
        repo_path.as_ref(),
        commit_hash,
        new_message,
        modify_all_branches,
        branch_name,
    )?;

    let status = child.wait()?;

    if !status.success() {
        anyhow::bail!("git filter-branch failed with status: {}", status);
    }

    let affected_commits = get_original_refs(repo_path.as_ref())?;
    let diff_output = get_git_log(repo_path.as_ref(), 10)?;

    Ok(PreviewData::new(
        commit_hash.to_string(),
        old_message,
        new_message.to_string(),
        affected_commits,
        diff_output,
    ))
}

pub fn rollback_changes<P: AsRef<Path>>(repo_path: P) -> Result<usize> {
    restore_original_refs(repo_path)
}

pub fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> Option<String> {
    GitRepositoryImpl::open(repo_path)
        .ok()
        .and_then(|repo| repo.get_current_branch())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_branch() {
        let current_dir = std::env::current_dir().unwrap();
        let _ = get_current_branch(current_dir);
    }
}

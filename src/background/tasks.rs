use crate::background::BackgroundMessage;
use crate::git::{self, RefUpdate, RewriteScope};
use crate::models::{LogLevel, PreviewData, RefChange, RewriteDetails};
use git2::{Oid, Repository};
use std::path::Path;
use std::sync::mpsc::Sender;

struct Reporter {
    tx: Sender<BackgroundMessage>,
}

impl Reporter {
    fn log(&self, message: impl Into<String>, level: LogLevel) {
        self.tx
            .send(BackgroundMessage::Log(message.into(), level))
            .ok();
    }

    fn info(&self, message: impl Into<String>) {
        self.log(message, LogLevel::Info);
    }

    fn success(&self, message: impl Into<String>) {
        self.log(message, LogLevel::Success);
    }

    fn progress(&self, value: f32) {
        self.tx.send(BackgroundMessage::Progress(value)).ok();
    }

    fn send(&self, message: BackgroundMessage) {
        self.tx.send(message).ok();
    }

    fn fail(&self, context: &str, error: anyhow::Error) {
        self.send(BackgroundMessage::Error(format!(
            "{}: {:#}",
            context, error
        )));
    }
}

pub fn load_commits_task(
    path: &Path,
    limit: usize,
    offset: usize,
    reset: bool,
    tx: Sender<BackgroundMessage>,
) {
    let reporter = Reporter { tx };

    let repo = match git::open(path) {
        Ok(repo) => repo,
        Err(e) => return reporter.fail("Failed to open repository", e),
    };

    if send_commits(&reporter, &repo, limit, offset, reset) && reset {
        report_pending_rewrite(&reporter, &repo);
    }
    reporter.send(BackgroundMessage::Done);
}

pub fn rewrite_commit_task(
    path: &Path,
    commit_hash: &str,
    new_message: &str,
    scope: RewriteScope,
    reload_limit: usize,
    tx: Sender<BackgroundMessage>,
) {
    let reporter = Reporter { tx };

    let result = (|| {
        let repo = git::open(path)?;
        let target = Oid::from_str(commit_hash)?;
        let old_message =
            String::from_utf8_lossy(repo.find_commit(target)?.message_bytes()).into_owned();

        reporter.info("Rewriting history...");
        let result = git::rewrite_message(&repo, target, new_message, scope, |done, total| {
            reporter.progress(done as f32 / total.max(1) as f32 * 0.9);
        })?;

        let details = RewriteDetails {
            old_hash: commit_hash.to_string(),
            new_hash: result.new_target.to_string(),
            old_message,
            new_message: git::normalize_message(new_message)?,
            rewritten_commits: result.rewritten_commits,
        };
        Ok::<_, anyhow::Error>((repo, details, result.ref_updates))
    })();

    let (repo, details, ref_updates) = match result {
        Ok(r) => r,
        Err(e) => return reporter.fail("Rewrite failed", e),
    };

    reporter.success(format!(
        "Rewrote {} commits, moved {} refs. Review and confirm the change",
        details.rewritten_commits,
        ref_updates.len()
    ));
    reporter.progress(1.0);
    reporter.send(BackgroundMessage::PreviewReady(PreviewData {
        details: Some(details),
        ref_changes: to_ref_changes(&ref_updates),
    }));
    send_commits(&reporter, &repo, reload_limit, 0, true);
    reporter.send(BackgroundMessage::Done);
}

pub fn confirm_rewrite_task(path: &Path, reload_limit: usize, tx: Sender<BackgroundMessage>) {
    let reporter = Reporter { tx };

    let repo = match git::open(path) {
        Ok(repo) => repo,
        Err(e) => return reporter.fail("Failed to open repository", e),
    };

    match git::confirm(&repo) {
        Ok(removed) => {
            reporter.success(format!("Changes confirmed, {} backups removed", removed));
            reporter.log(
                "Push rewritten branches with: git push --force-with-lease",
                LogLevel::Warning,
            );
        }
        Err(e) => return reporter.fail("Failed to confirm changes", e),
    }

    send_commits(&reporter, &repo, reload_limit, 0, true);
    reporter.send(BackgroundMessage::Done);
}

pub fn rollback_changes_task(path: &Path, reload_limit: usize, tx: Sender<BackgroundMessage>) {
    let reporter = Reporter { tx };

    let repo = match git::open(path) {
        Ok(repo) => repo,
        Err(e) => return reporter.fail("Failed to open repository", e),
    };

    reporter.info("Restoring refs from backup...");
    reporter.progress(0.5);

    match git::rollback(&repo) {
        Ok(restored) => {
            reporter.success(format!("Rollback completed, {} refs restored", restored));
            reporter.progress(1.0);
        }
        Err(e) => return reporter.fail("Rollback failed", e),
    }

    send_commits(&reporter, &repo, reload_limit, 0, true);
    reporter.send(BackgroundMessage::Done);
}

/// Sends a page of commits. Returns `false` if loading failed.
fn send_commits(
    reporter: &Reporter,
    repo: &Repository,
    limit: usize,
    offset: usize,
    reset: bool,
) -> bool {
    match git::load_commits(repo, limit, offset) {
        Ok(page) => {
            reporter.info(format!("Loaded {} commits", page.commits.len()));
            reporter.send(BackgroundMessage::CommitsLoaded {
                commits: page.commits,
                has_more: page.has_more,
                reset,
            });
            true
        }
        Err(e) => {
            reporter.fail("Error reading commits", e);
            false
        }
    }
}

fn report_pending_rewrite(reporter: &Reporter, repo: &Repository) {
    match git::pending_backups(repo) {
        Ok(updates) if !updates.is_empty() => {
            reporter.log(
                "Found an unconfirmed rewrite: confirm or roll it back",
                LogLevel::Warning,
            );
            reporter.send(BackgroundMessage::PreviewReady(PreviewData {
                details: None,
                ref_changes: to_ref_changes(&updates),
            }));
        }
        Ok(_) => {}
        Err(e) => reporter.fail("Failed to check for unconfirmed rewrite", e),
    }
}

fn to_ref_changes(updates: &[RefUpdate]) -> Vec<RefChange> {
    updates
        .iter()
        .map(|u| RefChange {
            name: u.name.clone(),
            old_hash: u.old_commit.to_string(),
            new_hash: u.new_commit.to_string(),
        })
        .collect()
}

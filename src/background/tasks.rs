use crate::background::BackgroundMessage;
use crate::git::{rollback_changes, GitRepository, GitRepositoryImpl};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub fn load_commits_task(
    path: PathBuf,
    limit: usize,
    offset: usize,
    reset: bool,
    tx: Sender<BackgroundMessage>,
) {
    if reset {
        tx.send(BackgroundMessage::Log("Opening repository...".to_string()))
            .ok();
    }

    let repo = match GitRepositoryImpl::open(&path) {
        Ok(r) => r,
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!(
                "Failed to open repository: {}",
                e
            )))
            .ok();
            return;
        }
    };

    if reset {
        tx.send(BackgroundMessage::Log("Reading commits...".to_string()))
            .ok();
    }

    match repo.load_commits(limit, offset) {
        Ok(commits) => {
            let count = commits.len();
            if reset {
                tx.send(BackgroundMessage::Log(format!(
                    "✅ Loaded {} commits",
                    count
                )))
                .ok();
            } else {
                tx.send(BackgroundMessage::Log(format!(
                    "✅ Loaded {} more commits",
                    count
                )))
                .ok();
            }
            tx.send(BackgroundMessage::CommitsLoaded(commits)).ok();
        }
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!(
                "Error reading commits: {}",
                e
            )))
            .ok();
        }
    }

    tx.send(BackgroundMessage::Done).ok();
}

pub fn rewrite_commit_task(
    path: PathBuf,
    commit_hash: String,
    new_message: String,
    modify_all_branches: bool,
    branch_name: Option<String>,
    tx: Sender<BackgroundMessage>,
) {
    let send_log = |msg: &str| {
        tx.send(BackgroundMessage::Log(msg.to_string())).ok();
    };

    send_log("Searching for commit in repository...");
    tx.send(BackgroundMessage::Progress(0.3)).ok();

    send_log("Preparing to rewrite history...");
    send_log("⚙️ Using git filter-branch...");
    send_log("⏳ This may take a while...");
    tx.send(BackgroundMessage::Progress(0.5)).ok();

    let mut child = match crate::git::commands::run_git_filter_branch(
        &path,
        &commit_hash,
        &new_message,
        modify_all_branches,
        branch_name.as_deref(),
    ) {
        Ok(c) => c,
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!(
                "Failed to start git filter-branch: {}",
                e
            )))
            .ok();
            return;
        }
    };

    // read stdout/stderr in separate threads
    let tx_stdout = tx.clone();
    let tx_stderr = tx.clone();

    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        tx_stdout
                            .send(BackgroundMessage::Log(format!("📝 {}", line)))
                            .ok();
                    }
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if !line.trim().is_empty() {
                        tx_stderr
                            .send(BackgroundMessage::Log(format!("📝 {}", line)))
                            .ok();
                    }
                }
            }
        });
    }

    // send progress updates while process runs
    let mut progress = 0.5;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        progress += 0.05;
        if progress > 0.9 {
            progress = 0.9;
        }
        tx.send(BackgroundMessage::Progress(progress)).ok();

        match child.try_wait() {
            Ok(Some(status)) => {
                tx.send(BackgroundMessage::Progress(0.9)).ok();

                if !status.success() {
                    tx.send(BackgroundMessage::Error(
                        "git filter-branch failed (see logs above)".to_string(),
                    ))
                    .ok();
                    return;
                }

                send_log("✅ git filter-branch completed successfully");
                break;
            }
            Ok(None) => {
                continue;
            }
            Err(e) => {
                tx.send(BackgroundMessage::Error(format!(
                    "Error checking status: {}",
                    e
                )))
                .ok();
                return;
            }
        }
    }

    send_log("Collecting preview data...");

    let repo = match git2::Repository::open(&path) {
        Ok(r) => r,
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!(
                "Failed to open repository: {}",
                e
            )))
            .ok();
            return;
        }
    };

    let target_oid = match git2::Oid::from_str(&commit_hash) {
        Ok(oid) => oid,
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!(
                "Invalid hash format: {}",
                e
            )))
            .ok();
            return;
        }
    };

    let target_commit = match repo.find_commit(target_oid) {
        Ok(c) => c,
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!("Commit not found: {}", e)))
                .ok();
            return;
        }
    };

    let old_message = target_commit
        .message()
        .unwrap_or("(no message)")
        .lines()
        .next()
        .unwrap_or("(no message)")
        .to_string();

    let affected_commits = match crate::git::commands::get_original_refs(&path) {
        Ok(refs) => refs,
        Err(_) => Vec::new(),
    };

    let diff_output = match crate::git::commands::get_git_log(&path, 10) {
        Ok(log) => log,
        Err(_) => "Failed to get diff".to_string(),
    };

    let preview_data = crate::models::PreviewData::new(
        commit_hash.clone(),
        old_message,
        new_message,
        affected_commits,
        diff_output,
    );

    send_log(&format!(
        "✅ Commit {} successfully rewritten! Showing preview...",
        &commit_hash[..8]
    ));
    tx.send(BackgroundMessage::Progress(1.0)).ok();
    tx.send(BackgroundMessage::PreviewReady(preview_data)).ok();
    tx.send(BackgroundMessage::Done).ok();
}

pub fn rollback_changes_task(path: PathBuf, tx: Sender<BackgroundMessage>) {
    let send_log = |msg: &str| {
        tx.send(BackgroundMessage::Log(msg.to_string())).ok();
    };

    send_log("Restoring original refs...");
    tx.send(BackgroundMessage::Progress(0.3)).ok();

    match rollback_changes(&path) {
        Ok(restored_count) => {
            tx.send(BackgroundMessage::Progress(0.7)).ok();
            send_log(&format!(
                "✅ Rollback completed! Restored {} refs",
                restored_count
            ));
        }
        Err(e) => {
            tx.send(BackgroundMessage::Error(format!("Rollback error: {}", e)))
                .ok();
        }
    }

    tx.send(BackgroundMessage::Progress(1.0)).ok();
    tx.send(BackgroundMessage::Done).ok();
}

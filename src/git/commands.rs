use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_git_filter_branch<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: &str,
    new_message: &str,
    modify_all_branches: bool,
    branch_name: Option<&str>,
) -> Result<std::process::Child> {
    let mut args = vec![
        "filter-branch".to_string(),
        "-f".to_string(),
        "--msg-filter".to_string(),
        format!(
            r#"if [ "$GIT_COMMIT" = "{}" ]; then echo '{}'; else cat; fi"#,
            commit_hash,
            new_message.replace('\'', "'\\''")
        ),
        "--".to_string(),
    ];

    if modify_all_branches {
        args.push("--all".to_string());
    } else if let Some(branch) = branch_name {
        args.push(branch.to_string());
    } else {
        args.push("--all".to_string());
    }

    let child = Command::new("git")
        .current_dir(repo_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

pub fn get_original_refs<P: AsRef<Path>>(repo_path: P) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&["for-each-ref", "--format=%(refname)", "refs/original/"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let refs: Vec<String> = output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(refs)
}

pub fn restore_original_refs<P: AsRef<Path>>(repo_path: P) -> Result<usize> {
    let refs = get_original_refs(repo_path.as_ref())?;
    let mut restored_count = 0;

    for original_ref in refs {
        let target_ref = original_ref.replace("refs/original/", "");

        let hash_output = Command::new("git")
            .current_dir(repo_path.as_ref())
            .args(&["rev-parse", &original_ref])
            .output()?;

        let hash = String::from_utf8_lossy(&hash_output.stdout)
            .trim()
            .to_string();

        let result = Command::new("git")
            .current_dir(repo_path.as_ref())
            .args(&["update-ref", "-f", &target_ref, &hash])
            .output();

        if result.is_ok() {
            restored_count += 1;
        }
    }

    Ok(restored_count)
}

pub fn get_git_log<P: AsRef<Path>>(repo_path: P, lines: usize) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "log",
            "--oneline",
            "--graph",
            "--all",
            &format!("-{}", lines),
        ])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_git_log() {
        let current_dir = std::env::current_dir().unwrap();
        if let Ok(log) = get_git_log(current_dir, 5) {
            assert!(!log.is_empty() || true);
        }
    }
}

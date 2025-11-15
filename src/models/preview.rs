#[derive(Clone, Debug)]
pub struct PreviewData {
    pub commit_hash: String,
    pub old_message: String,
    pub new_message: String,
    pub affected_commits: Vec<String>,
    pub diff_output: String,
}

impl PreviewData {
    pub fn new(
        commit_hash: String,
        old_message: String,
        new_message: String,
        affected_commits: Vec<String>,
        diff_output: String,
    ) -> Self {
        Self {
            commit_hash,
            old_message,
            new_message,
            affected_commits,
            diff_output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_data_creation() {
        let preview = PreviewData::new(
            "abc123".to_string(),
            "Old message".to_string(),
            "New message".to_string(),
            vec!["refs/heads/main".to_string()],
            "* abc123 New message".to_string(),
        );

        assert_eq!(preview.commit_hash, "abc123");
        assert_eq!(preview.old_message, "Old message");
        assert_eq!(preview.new_message, "New message");
        assert_eq!(preview.affected_commits.len(), 1);
        assert_eq!(preview.diff_output, "* abc123 New message");
    }
}

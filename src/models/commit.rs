#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

impl CommitInfo {
    pub fn new(
        hash: String,
        short_hash: String,
        message: String,
        date: String,
        author: String,
    ) -> Self {
        Self {
            hash,
            short_hash,
            message,
            date,
            author,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_info_creation() {
        let commit = CommitInfo::new(
            "abc123def456".to_string(),
            "abc123de".to_string(),
            "Initial commit".to_string(),
            "2024-01-01 12:00".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(commit.hash, "abc123def456");
        assert_eq!(commit.short_hash, "abc123de");
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.date, "2024-01-01 12:00");
        assert_eq!(commit.author, "John Doe");
    }
}

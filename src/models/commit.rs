#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    /// First line of the message.
    pub summary: String,
    /// Full message including body and trailers.
    pub message: String,
    pub date: String,
    pub author: String,
}

impl CommitInfo {
    pub fn new(hash: String, message: String, date: String, author: String) -> Self {
        let short_hash = hash.chars().take(8).collect();
        let summary = message.lines().next().unwrap_or("").to_string();
        Self {
            hash,
            short_hash,
            summary,
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
            "Initial commit\n\nLonger description\n".to_string(),
            "2024-01-01 12:00".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(commit.hash, "abc123def456");
        assert_eq!(commit.short_hash, "abc123de");
        assert_eq!(commit.summary, "Initial commit");
        assert_eq!(commit.message, "Initial commit\n\nLonger description\n");
        assert_eq!(commit.date, "2024-01-01 12:00");
        assert_eq!(commit.author, "John Doe");
    }
}

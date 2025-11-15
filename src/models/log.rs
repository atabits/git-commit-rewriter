#[derive(Clone, Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "📝",
            LogLevel::Success => "✅",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Debug => "🔍",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            LogLevel::Info => (150, 150, 160),
            LogLevel::Success => (80, 200, 120),
            LogLevel::Warning => (255, 180, 50),
            LogLevel::Error => (255, 100, 100),
            LogLevel::Debug => (120, 120, 140),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LogFilter {
    All,
    Important,
    ErrorsOnly,
}

impl LogFilter {
    pub fn should_show(&self, level: &LogLevel) -> bool {
        match self {
            LogFilter::All => true,
            LogFilter::Important => matches!(
                level,
                LogLevel::Success | LogLevel::Warning | LogLevel::Error
            ),
            LogFilter::ErrorsOnly => matches!(level, LogLevel::Error),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LogFilter::All => "All",
            LogFilter::Important => "Important",
            LogFilter::ErrorsOnly => "Errors",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
    pub level: LogLevel,
}

impl LogEntry {
    pub fn new(timestamp: String, message: String, level: LogLevel) -> Self {
        Self {
            timestamp,
            message,
            level,
        }
    }

    pub fn now(message: String, level: LogLevel) -> Self {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        Self::new(timestamp, message, level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_icon() {
        assert_eq!(LogLevel::Info.icon(), "📝");
        assert_eq!(LogLevel::Success.icon(), "✅");
        assert_eq!(LogLevel::Warning.icon(), "⚠️");
        assert_eq!(LogLevel::Error.icon(), "❌");
        assert_eq!(LogLevel::Debug.icon(), "🔍");
    }

    #[test]
    fn test_log_filter_should_show() {
        let filter = LogFilter::Important;
        assert!(!filter.should_show(&LogLevel::Info));
        assert!(filter.should_show(&LogLevel::Success));
        assert!(filter.should_show(&LogLevel::Warning));
        assert!(filter.should_show(&LogLevel::Error));
        assert!(!filter.should_show(&LogLevel::Debug));
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            "12:00:00".to_string(),
            "Test message".to_string(),
            LogLevel::Info,
        );

        assert_eq!(entry.timestamp, "12:00:00");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
    }
}

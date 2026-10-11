use crate::models::{CommitInfo, LogLevel, PreviewData};

pub enum BackgroundMessage {
    Log(String, LogLevel),
    Progress(f32),
    CommitsLoaded {
        commits: Vec<CommitInfo>,
        has_more: bool,
        reset: bool,
    },
    /// A rewrite is waiting to be confirmed or rolled back.
    PreviewReady(PreviewData),
    Error(String),
    Done,
}

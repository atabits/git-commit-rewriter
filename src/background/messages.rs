use crate::models::{CommitInfo, LogLevel, PreviewData};

#[derive(Clone)]
pub enum BackgroundMessage {
    Log(String),
    LogTyped(String, LogLevel),
    Progress(f32),
    CommitsLoaded(Vec<CommitInfo>),
    CommitRewritten,
    PreviewReady(PreviewData),
    Error(String),
    Done,
}

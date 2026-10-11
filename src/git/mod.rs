pub mod repository;
pub mod rewrite;

pub use repository::{current_branch, is_valid_repo, load_commits, open, CommitPage};
pub use rewrite::{
    confirm, normalize_message, pending_backups, rewrite_message, rollback, RefUpdate,
    RewriteResult, RewriteScope, BACKUP_PREFIX,
};

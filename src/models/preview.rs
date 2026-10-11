/// A ref moved by a rewrite, for display.
#[derive(Clone, Debug, PartialEq)]
pub struct RefChange {
    pub name: String,
    pub old_hash: String,
    pub new_hash: String,
}

/// What the user reviews before confirming or rolling back a rewrite.
#[derive(Clone, Debug)]
pub struct PreviewData {
    /// Details of the rewrite just performed. `None` when an unconfirmed
    /// rewrite was found on opening the repository.
    pub details: Option<RewriteDetails>,
    pub ref_changes: Vec<RefChange>,
}

#[derive(Clone, Debug)]
pub struct RewriteDetails {
    pub old_hash: String,
    pub new_hash: String,
    pub old_message: String,
    pub new_message: String,
    pub rewritten_commits: usize,
}

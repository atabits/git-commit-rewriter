//! Rewriting a commit message directly through libgit2.
//!
//! The target commit gets the new message and every descendant reachable from
//! the selected refs is recreated with updated parents. Trees, authors and
//! committers (including timestamps) are preserved. Before any ref is moved its
//! original value is saved under [`BACKUP_PREFIX`], so the change can be
//! confirmed (backups deleted) or rolled back (refs restored from backups).

use anyhow::{bail, Context, Result};
use git2::{ObjectType, Oid, Repository, Signature, Sort};
use std::collections::HashMap;

/// Namespace for backups of refs moved by an unconfirmed rewrite.
pub const BACKUP_PREFIX: &str = "refs/commit-rewriter/backup/";

const HEAD: &str = "HEAD";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteScope {
    /// Only the checked-out branch (or detached HEAD).
    CurrentBranch,
    /// All local branches, tags and detached HEAD.
    AllBranches,
}

/// A ref moved by a rewrite, with the commits it pointed to before and after.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefUpdate {
    pub name: String,
    pub old_commit: Oid,
    pub new_commit: Oid,
}

#[derive(Clone, Debug)]
pub struct RewriteResult {
    pub new_target: Oid,
    pub rewritten_commits: usize,
    pub ref_updates: Vec<RefUpdate>,
}

#[derive(Clone, Copy, Debug)]
enum RefKind {
    /// Branch or lightweight tag pointing straight at a commit.
    Direct,
    /// Annotated tag object pointing at a commit.
    AnnotatedTag(Oid),
    DetachedHead,
}

#[derive(Clone, Debug)]
struct TrackedRef {
    name: String,
    commit: Oid,
    kind: RefKind,
}

impl TrackedRef {
    /// Object the ref itself points to, which is what a backup must keep.
    fn raw_target(&self) -> Oid {
        match self.kind {
            RefKind::AnnotatedTag(tag) => tag,
            RefKind::Direct | RefKind::DetachedHead => self.commit,
        }
    }
}

/// Normalizes a message the way `git commit` does: trims trailing whitespace,
/// collapses blank lines and ends it with a newline.
pub fn normalize_message(message: &str) -> Result<String> {
    Ok(git2::message_prettify(message, None)?)
}

/// Replaces the message of `target` and rewrites its descendants.
///
/// `progress` is called with `(processed, total)` commits.
pub fn rewrite_message(
    repo: &Repository,
    target: Oid,
    new_message: &str,
    scope: RewriteScope,
    mut progress: impl FnMut(usize, usize),
) -> Result<RewriteResult> {
    if !pending_backups(repo)?.is_empty() {
        bail!("an unconfirmed rewrite already exists: confirm or roll it back first");
    }

    let message = normalize_message(new_message)?;
    if message.trim().is_empty() {
        bail!("commit message must not be empty");
    }

    let target_commit = repo
        .find_commit(target)
        .with_context(|| format!("commit {} not found", target))?;
    if target_commit.message_bytes() == message.as_bytes() {
        bail!("new message is the same as the current one");
    }

    let refs: Vec<TrackedRef> = collect_refs(repo, scope)?
        .into_iter()
        .filter(|r| {
            r.commit == target || repo.graph_descendant_of(r.commit, target).unwrap_or(false)
        })
        .collect();
    if refs.is_empty() {
        bail!(
            "commit {} is not reachable from the selected {}",
            short(target),
            match scope {
                RewriteScope::CurrentBranch => "branch",
                RewriteScope::AllBranches => "branches and tags",
            }
        );
    }

    let mut walk = repo.revwalk()?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
    for r in &refs {
        walk.push(r.commit)?;
    }
    for parent in target_commit.parent_ids() {
        walk.hide(parent)?;
    }
    let oids = walk.collect::<Result<Vec<_>, _>>()?;
    let total = oids.len();

    let mut mapping: HashMap<Oid, Oid> = HashMap::new();
    for (i, oid) in oids.iter().enumerate() {
        let commit = repo.find_commit(*oid)?;
        let old_parents: Vec<Oid> = commit.parent_ids().collect();
        let new_parents: Vec<Oid> = old_parents
            .iter()
            .map(|p| *mapping.get(p).unwrap_or(p))
            .collect();

        if *oid == target || new_parents != old_parents {
            let parents = new_parents
                .iter()
                .map(|p| repo.find_commit(*p))
                .collect::<Result<Vec<_>, _>>()?;
            let parent_refs: Vec<_> = parents.iter().collect();
            let commit_message = if *oid == target {
                message.clone()
            } else {
                String::from_utf8_lossy(commit.message_raw_bytes()).into_owned()
            };
            let new_oid = repo.commit(
                None,
                &commit.author(),
                &commit.committer(),
                &commit_message,
                &commit.tree()?,
                &parent_refs,
            )?;
            mapping.insert(*oid, new_oid);
        }

        progress(i + 1, total);
    }

    let new_target = *mapping
        .get(&target)
        .context("target commit was not rewritten")?;

    let changed: Vec<&TrackedRef> = refs
        .iter()
        .filter(|r| mapping.contains_key(&r.commit))
        .collect();

    for r in &changed {
        repo.reference(
            &backup_name(&r.name),
            r.raw_target(),
            true,
            "commit-rewriter: backup",
        )?;
    }

    let mut ref_updates = Vec::new();
    for r in &changed {
        let new_commit = mapping[&r.commit];
        if let Err(e) = move_ref(repo, r, new_commit) {
            let rollback = rollback(repo)
                .map(|n| format!("{} refs restored", n))
                .unwrap_or_else(|err| format!("rollback failed: {:#}", err));
            bail!("failed to update {}: {:#} ({})", r.name, e, rollback);
        }
        ref_updates.push(RefUpdate {
            name: r.name.clone(),
            old_commit: r.commit,
            new_commit,
        });
    }

    Ok(RewriteResult {
        new_target,
        rewritten_commits: mapping.len(),
        ref_updates,
    })
}

/// Refs saved by an unconfirmed rewrite, with their backed-up and current commits.
pub fn pending_backups(repo: &Repository) -> Result<Vec<RefUpdate>> {
    let mut result = Vec::new();
    for reference in repo.references_glob(&format!("{}*", BACKUP_PREFIX))? {
        let reference = reference?;
        let backup = reference.name()?.to_string();
        let name = backup[BACKUP_PREFIX.len()..].to_string();
        let old_commit = reference.peel_to_commit()?.id();
        let new_commit = if name == HEAD {
            repo.head()?.peel_to_commit()?.id()
        } else {
            repo.find_reference(&name)?.peel_to_commit()?.id()
        };
        result.push(RefUpdate {
            name,
            old_commit,
            new_commit,
        });
    }
    Ok(result)
}

/// Accepts the rewrite by deleting its backups. Returns the number of backups removed.
pub fn confirm(repo: &Repository) -> Result<usize> {
    let mut removed = 0;
    for reference in repo.references_glob(&format!("{}*", BACKUP_PREFIX))? {
        reference?.delete()?;
        removed += 1;
    }
    Ok(removed)
}

/// Restores every ref saved by an unconfirmed rewrite. Returns the number of refs restored.
///
/// Each backup is deleted only after its ref was restored, so a failed rollback
/// can be retried.
pub fn rollback(repo: &Repository) -> Result<usize> {
    let mut restored = 0;
    let mut errors = Vec::new();

    let backups = repo
        .references_glob(&format!("{}*", BACKUP_PREFIX))?
        .collect::<Result<Vec<_>, _>>()?;

    for mut backup in backups {
        let backup_ref = backup.name()?.to_string();
        let name = backup_ref[BACKUP_PREFIX.len()..].to_string();
        let result = backup
            .target()
            .context("backup is not a direct reference")
            .and_then(|oid| {
                if name == HEAD {
                    repo.set_head_detached(oid)?;
                } else {
                    repo.reference(&name, oid, true, "commit-rewriter: rollback")?;
                }
                backup.delete()?;
                Ok(())
            });

        match result {
            Ok(()) => restored += 1,
            Err(e) => errors.push(format!("{}: {:#}", name, e)),
        }
    }

    if !errors.is_empty() {
        bail!(
            "restored {} refs, failed to restore {}: {}",
            restored,
            errors.len(),
            errors.join("; ")
        );
    }
    Ok(restored)
}

fn collect_refs(repo: &Repository, scope: RewriteScope) -> Result<Vec<TrackedRef>> {
    let mut refs = Vec::new();

    if repo.head_detached()? {
        let commit = repo.head()?.peel_to_commit()?.id();
        refs.push(TrackedRef {
            name: HEAD.to_string(),
            commit,
            kind: RefKind::DetachedHead,
        });
    } else if scope == RewriteScope::CurrentBranch {
        let head = repo.head().context("HEAD does not point to a commit")?;
        refs.push(TrackedRef {
            name: head.name()?.to_string(),
            commit: head.peel_to_commit()?.id(),
            kind: RefKind::Direct,
        });
    }

    if scope == RewriteScope::AllBranches {
        for glob in ["refs/heads/*", "refs/tags/*"] {
            for reference in repo.references_glob(glob)? {
                let reference = reference?;
                let Some(target) = reference.target() else {
                    continue;
                };
                let name = reference.name()?.to_string();
                let object = repo.find_object(target, None)?;
                match object.kind() {
                    Some(ObjectType::Commit) => refs.push(TrackedRef {
                        name,
                        commit: target,
                        kind: RefKind::Direct,
                    }),
                    Some(ObjectType::Tag) => {
                        let tag = object.peel_to_tag()?;
                        // Tags of tags and tags of non-commits are left as is.
                        if tag.target_type() == Some(ObjectType::Commit) {
                            refs.push(TrackedRef {
                                name,
                                commit: tag.target_id(),
                                kind: RefKind::AnnotatedTag(target),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(refs)
}

fn move_ref(repo: &Repository, r: &TrackedRef, new_commit: Oid) -> Result<()> {
    let log_message = format!("commit-rewriter: reword {}", short(r.commit));
    match r.kind {
        RefKind::Direct => {
            repo.reference(&r.name, new_commit, true, &log_message)?;
        }
        RefKind::DetachedHead => {
            repo.set_head_detached(new_commit)?;
        }
        RefKind::AnnotatedTag(tag_oid) => {
            let tag = repo.find_tag(tag_oid)?;
            let tagger = match tag.tagger() {
                Some(tagger) => tagger.to_owned(),
                None => repo
                    .signature()
                    .or_else(|_| Signature::now("unknown", "unknown"))?,
            };
            let target = repo.find_object(new_commit, Some(ObjectType::Commit))?;
            let new_tag = repo.tag_annotation_create(
                tag.name()?,
                &target,
                &tagger,
                &String::from_utf8_lossy(tag.message_bytes().unwrap_or_default()),
            )?;
            repo.reference(&r.name, new_tag, true, &log_message)?;
        }
    }
    Ok(())
}

fn backup_name(ref_name: &str) -> String {
    format!("{}{}", BACKUP_PREFIX, ref_name)
}

fn short(oid: Oid) -> String {
    oid.to_string()[..8].to_string()
}

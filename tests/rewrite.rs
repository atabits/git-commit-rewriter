use commit_rewriter::git::{self, RewriteScope, BACKUP_PREFIX};
use git2::{Oid, Repository, Signature, Time};
use tempfile::TempDir;

struct TestRepo {
    _dir: TempDir,
    repo: Repository,
    counter: i64,
}

impl TestRepo {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        repo.set_head("refs/heads/main").unwrap();
        Self {
            _dir: dir,
            repo,
            counter: 0,
        }
    }

    fn signature(&self, name: &str) -> Signature<'static> {
        Signature::new(
            name,
            &format!("{}@example.com", name.to_lowercase()),
            &Time::new(1_700_000_000 + self.counter * 60, 0),
        )
        .unwrap()
    }

    /// Creates a commit with a new file on top of `parents` and moves `ref_name` to it.
    fn commit(&mut self, ref_name: Option<&str>, message: &str, parents: &[Oid]) -> Oid {
        self.counter += 1;
        let blob = self
            .repo
            .blob(format!("content {}", self.counter).as_bytes())
            .unwrap();

        let mut builder = match parents.first() {
            Some(p) => {
                let tree = self.repo.find_commit(*p).unwrap().tree().unwrap();
                self.repo.treebuilder(Some(&tree)).unwrap()
            }
            None => self.repo.treebuilder(None).unwrap(),
        };
        builder
            .insert(format!("file{}.txt", self.counter), blob, 0o100644)
            .unwrap();
        let tree = self.repo.find_tree(builder.write().unwrap()).unwrap();

        let parents: Vec<_> = parents
            .iter()
            .map(|p| self.repo.find_commit(*p).unwrap())
            .collect();
        let parent_refs: Vec<_> = parents.iter().collect();
        let author = self.signature("Author");
        let committer = self.signature("Committer");
        self.repo
            .commit(ref_name, &author, &committer, message, &tree, &parent_refs)
            .unwrap()
    }

    fn ref_commit(&self, name: &str) -> Oid {
        self.repo
            .find_reference(name)
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
    }

    fn message(&self, oid: Oid) -> String {
        self.repo
            .find_commit(oid)
            .unwrap()
            .message()
            .unwrap()
            .to_string()
    }

    fn parent(&self, oid: Oid) -> Oid {
        self.repo.find_commit(oid).unwrap().parent_id(0).unwrap()
    }

    fn backups(&self) -> usize {
        self.repo
            .references_glob(&format!("{}*", BACKUP_PREFIX))
            .unwrap()
            .count()
    }
}

/// main: a -> b -> c, where b has a body and trailers.
fn linear_repo() -> (TestRepo, [Oid; 3]) {
    let mut t = TestRepo::new();
    let a = t.commit(Some("HEAD"), "First\n", &[]);
    let b = t.commit(
        Some("HEAD"),
        "Second\n\nDetailed body.\n\nSigned-off-by: Author <author@example.com>\n",
        &[a],
    );
    let c = t.commit(Some("HEAD"), "Third\n", &[b]);
    (t, [a, b, c])
}

fn rewrite(t: &TestRepo, target: Oid, message: &str, scope: RewriteScope) -> git::RewriteResult {
    git::rewrite_message(&t.repo, target, message, scope, |_, _| {}).unwrap()
}

#[test]
fn rewrites_message_and_keeps_body_trees_and_signatures() {
    let (t, [a, b, c]) = linear_repo();
    let old = t.repo.find_commit(b).unwrap();
    let new_message = old
        .message()
        .unwrap()
        .replacen("Second", "Second, reworded", 1);

    let result = rewrite(&t, b, &new_message, RewriteScope::CurrentBranch);

    let new_c = t.ref_commit("refs/heads/main");
    let new_b = t.parent(new_c);
    assert_ne!(new_c, c);
    assert_eq!(new_b, result.new_target);
    assert_eq!(
        t.parent(new_b),
        a,
        "commits before the target are untouched"
    );
    assert_eq!(result.rewritten_commits, 2);

    assert_eq!(
        t.message(new_b),
        "Second, reworded\n\nDetailed body.\n\nSigned-off-by: Author <author@example.com>\n"
    );
    assert_eq!(t.message(new_c), "Third\n");

    for (old_oid, new_oid) in [(b, new_b), (c, new_c)] {
        let old = t.repo.find_commit(old_oid).unwrap();
        let new = t.repo.find_commit(new_oid).unwrap();
        assert_eq!(old.tree_id(), new.tree_id());
        assert_eq!(old.author().to_string(), new.author().to_string());
        assert_eq!(old.author().when(), new.author().when());
        assert_eq!(old.committer().when(), new.committer().when());
    }
}

#[test]
fn messages_are_passed_verbatim() {
    let (t, [_, b, _]) = linear_repo();
    let message = "Quote ' \"double\" $HOME `cmd` back\\slash \\n \\c";

    let result = rewrite(&t, b, message, RewriteScope::CurrentBranch);

    assert_eq!(t.message(result.new_target), format!("{}\n", message));
}

#[test]
fn current_branch_scope_leaves_other_refs() {
    let (mut t, [_, b, c]) = linear_repo();
    let feature = t.commit(Some("refs/heads/feature"), "Feature\n", &[c]);
    t.repo
        .tag_lightweight("v1", &t.repo.find_object(c, None).unwrap(), false)
        .unwrap();

    let result = rewrite(&t, b, "Reworded\n", RewriteScope::CurrentBranch);

    assert_eq!(result.ref_updates.len(), 1);
    assert_eq!(result.ref_updates[0].name, "refs/heads/main");
    assert_eq!(t.ref_commit("refs/heads/feature"), feature);
    assert_eq!(t.ref_commit("refs/tags/v1"), c);
}

#[test]
fn all_branches_scope_updates_branches_and_tags() {
    let (mut t, [a, b, c]) = linear_repo();
    let feature = t.commit(Some("refs/heads/feature"), "Feature\n", &[c]);
    let unrelated = t.commit(Some("refs/heads/old"), "Old branch\n", &[a]);
    t.repo
        .tag_lightweight("light", &t.repo.find_object(c, None).unwrap(), false)
        .unwrap();
    let tagger = t.signature("Tagger");
    t.repo
        .tag(
            "annotated",
            &t.repo.find_object(b, None).unwrap(),
            &tagger,
            "Release notes\n",
            false,
        )
        .unwrap();

    let result = rewrite(&t, b, "Reworded\n", RewriteScope::AllBranches);

    let new_c = t.ref_commit("refs/heads/main");
    let new_b = result.new_target;
    assert_eq!(t.parent(new_c), new_b);
    assert_eq!(t.ref_commit("refs/tags/light"), new_c);
    assert_eq!(t.ref_commit("refs/tags/annotated"), new_b);
    assert_eq!(t.parent(t.ref_commit("refs/heads/feature")), new_c);
    assert_ne!(t.ref_commit("refs/heads/feature"), feature);
    assert_eq!(
        t.ref_commit("refs/heads/old"),
        unrelated,
        "branch without the commit is untouched"
    );

    let tag = t
        .repo
        .find_reference("refs/tags/annotated")
        .unwrap()
        .peel_to_tag()
        .unwrap();
    assert_eq!(tag.message().unwrap(), Some("Release notes\n"));
    assert_eq!(tag.tagger().unwrap().name().unwrap(), "Tagger");

    let mut names: Vec<_> = result.ref_updates.iter().map(|u| u.name.as_str()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "refs/heads/feature",
            "refs/heads/main",
            "refs/tags/annotated",
            "refs/tags/light"
        ]
    );
}

#[test]
fn rewrites_merge_commits() {
    let mut t = TestRepo::new();
    let a = t.commit(Some("HEAD"), "Base\n", &[]);
    let side = t.commit(None, "Side\n", &[a]);
    let main = t.commit(Some("HEAD"), "Main\n", &[a]);
    let merge = t.commit(Some("HEAD"), "Merge\n", &[main, side]);

    rewrite(&t, side, "Side reworded\n", RewriteScope::CurrentBranch);

    let new_merge = t.ref_commit("refs/heads/main");
    assert_ne!(new_merge, merge);
    let new_merge = t.repo.find_commit(new_merge).unwrap();
    assert_eq!(
        new_merge.parent_id(0).unwrap(),
        main,
        "first parent was not rewritten"
    );
    assert_eq!(
        t.message(new_merge.parent_id(1).unwrap()),
        "Side reworded\n"
    );
}

#[test]
fn rewrites_root_commit() {
    let (t, [a, _, _]) = linear_repo();

    let result = rewrite(&t, a, "Root reworded\n", RewriteScope::CurrentBranch);

    let root = t.repo.find_commit(result.new_target).unwrap();
    assert_eq!(root.parent_count(), 0);
    assert_eq!(t.message(root.id()), "Root reworded\n");
    assert_eq!(result.rewritten_commits, 3);
}

#[test]
fn rewrites_detached_head() {
    let (t, [_, b, c]) = linear_repo();
    t.repo.set_head_detached(c).unwrap();

    rewrite(&t, b, "Reworded\n", RewriteScope::CurrentBranch);

    assert!(t.repo.head_detached().unwrap());
    let head = t.repo.head().unwrap().peel_to_commit().unwrap().id();
    assert_ne!(head, c);
    assert_eq!(t.message(t.parent(head)), "Reworded\n");
    assert_eq!(
        t.ref_commit("refs/heads/main"),
        c,
        "branch is not part of current scope"
    );
}

#[test]
fn rollback_restores_all_refs_and_removes_backups() {
    let (mut t, [_, b, c]) = linear_repo();
    t.commit(Some("refs/heads/feature"), "Feature\n", &[c]);
    let tagger = t.signature("Tagger");
    let tag = t
        .repo
        .tag(
            "annotated",
            &t.repo.find_object(b, None).unwrap(),
            &tagger,
            "Notes\n",
            false,
        )
        .unwrap();
    let before: Vec<_> = [
        "refs/heads/main",
        "refs/heads/feature",
        "refs/tags/annotated",
    ]
    .iter()
    .map(|name| t.repo.find_reference(name).unwrap().target().unwrap())
    .collect();

    rewrite(&t, b, "Reworded\n", RewriteScope::AllBranches);
    assert_eq!(t.backups(), 3);
    assert_eq!(git::pending_backups(&t.repo).unwrap().len(), 3);

    assert_eq!(git::rollback(&t.repo).unwrap(), 3);

    let after: Vec<_> = [
        "refs/heads/main",
        "refs/heads/feature",
        "refs/tags/annotated",
    ]
    .iter()
    .map(|name| t.repo.find_reference(name).unwrap().target().unwrap())
    .collect();
    assert_eq!(before, after);
    assert_eq!(
        after[2], tag,
        "annotated tag points to the original tag object"
    );
    assert_eq!(t.backups(), 0);
}

#[test]
fn confirm_removes_backups_and_allows_next_rewrite() {
    let (t, [a, b, _]) = linear_repo();

    rewrite(&t, b, "Reworded\n", RewriteScope::CurrentBranch);
    assert_eq!(git::confirm(&t.repo).unwrap(), 1);
    assert_eq!(t.backups(), 0);
    assert!(git::pending_backups(&t.repo).unwrap().is_empty());

    rewrite(&t, a, "First reworded\n", RewriteScope::CurrentBranch);
    // rolling back the second rewrite must not undo the confirmed first one
    git::rollback(&t.repo).unwrap();
    let head = t.ref_commit("refs/heads/main");
    assert_eq!(t.message(t.parent(head)), "Reworded\n");
}

#[test]
fn pending_rewrite_blocks_new_one() {
    let (t, [a, b, _]) = linear_repo();
    rewrite(&t, b, "Reworded\n", RewriteScope::CurrentBranch);

    let err = git::rewrite_message(
        &t.repo,
        a,
        "Other\n",
        RewriteScope::CurrentBranch,
        |_, _| {},
    )
    .unwrap_err();
    assert!(err.to_string().contains("unconfirmed rewrite"), "{}", err);
}

#[test]
fn rejects_unchanged_empty_and_unreachable() {
    let (mut t, [a, b, _]) = linear_repo();
    let other = t.commit(Some("refs/heads/other"), "Other\n", &[a]);

    let unchanged = t.message(b);
    for (target, message, expected) in [
        (b, unchanged.as_str(), "same as the current"),
        (b, "  \n\n", "must not be empty"),
        (other, "Reworded\n", "not reachable"),
    ] {
        let err = git::rewrite_message(
            &t.repo,
            target,
            message,
            RewriteScope::CurrentBranch,
            |_, _| {},
        )
        .unwrap_err();
        assert!(err.to_string().contains(expected), "{}", err);
    }
    assert_eq!(t.backups(), 0);
}

#[test]
fn reports_progress_for_every_commit() {
    let (t, [_, b, _]) = linear_repo();
    let mut calls = Vec::new();

    git::rewrite_message(
        &t.repo,
        b,
        "Reworded\n",
        RewriteScope::CurrentBranch,
        |done, total| calls.push((done, total)),
    )
    .unwrap();

    assert_eq!(calls, vec![(1, 2), (2, 2)]);
}

#[test]
fn loads_commits_page_by_page() {
    let mut t = TestRepo::new();
    let mut parent = Vec::new();
    for i in 0..5 {
        parent = vec![t.commit(
            Some("HEAD"),
            &format!("Commit {}\n\nBody {}\n", i, i),
            &parent,
        )];
    }

    let page = git::load_commits(&t.repo, 3, 0).unwrap();
    assert_eq!(page.commits.len(), 3);
    assert!(page.has_more);
    assert_eq!(page.commits[0].summary, "Commit 4");
    assert_eq!(page.commits[0].message, "Commit 4\n\nBody 4\n");

    let page = git::load_commits(&t.repo, 3, 3).unwrap();
    assert_eq!(page.commits.len(), 2);
    assert!(!page.has_more);

    let page = git::load_commits(&t.repo, 5, 0).unwrap();
    assert!(!page.has_more, "exactly limit commits means no more pages");
}

#[test]
fn empty_repository_has_no_commits() {
    let t = TestRepo::new();
    let page = git::load_commits(&t.repo, 50, 0).unwrap();
    assert!(page.commits.is_empty());
    assert!(!page.has_more);
    assert_eq!(git::current_branch(&t.repo), None);
}

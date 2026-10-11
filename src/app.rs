use crate::background::{
    confirm_rewrite_task, load_commits_task, rewrite_commit_task, rollback_changes_task,
    BackgroundMessage,
};
use crate::git::{self, RewriteScope};
use crate::models::{CommitInfo, LogEntry, LogFilter, LogLevel, PreviewData};
use crate::ui::{commits_list, editor_panel, logs_panel, main_window, preview_modal, theme};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};

/// Number of commits fetched per page.
const PAGE_SIZE: usize = 50;
const MAX_LOG_ENTRIES: usize = 1000;

pub struct CommitRewriterApp {
    pub repo_path: Option<PathBuf>,
    pub current_branch: Option<String>,
    pub modify_all_branches: bool,

    pub commits: Vec<CommitInfo>,
    pub selected_index: Option<usize>,
    pub loading_more: bool,
    pub has_more_commits: bool,

    pub new_message: String,

    pub logs: Vec<LogEntry>,
    pub log_filter: LogFilter,

    pub progress: f32,
    pub show_progress: bool,
    pub is_processing: bool,

    pub message_receiver: Option<mpsc::Receiver<BackgroundMessage>>,
    /// Result of an open folder dialog; `None` inside means it was cancelled.
    folder_receiver: Option<mpsc::Receiver<Option<PathBuf>>>,
    initialized: bool,

    pub show_help: bool,
    pub search_query: String,

    pub preview_data: Option<PreviewData>,
}

impl Default for CommitRewriterApp {
    fn default() -> Self {
        let mut app = Self {
            repo_path: None,
            current_branch: None,
            modify_all_branches: false,

            commits: Vec::new(),
            selected_index: None,
            loading_more: false,
            has_more_commits: false,

            new_message: String::new(),

            logs: Vec::new(),
            log_filter: LogFilter::All,

            progress: 0.0,
            show_progress: false,
            is_processing: false,

            message_receiver: None,
            folder_receiver: None,
            initialized: false,

            show_help: false,
            search_query: String::new(),

            preview_data: None,
        };

        // try to use current directory if it's a git repo
        match std::env::current_dir() {
            Ok(dir) if git::is_valid_repo(&dir) => {
                app.add_log(&format!("Opened repository {}", theme::display_path(&dir)));
                app.set_repo(dir);
            }
            _ => app.add_log("Select a Git repository to get started"),
        }

        app
    }
}

impl CommitRewriterApp {
    pub fn add_log(&mut self, message: &str) {
        self.add_log_typed(message, LogLevel::Info);
    }

    pub fn add_log_typed(&mut self, message: &str, level: LogLevel) {
        self.logs.push(LogEntry::now(message.to_string(), level));

        // keep log size reasonable
        if self.logs.len() > MAX_LOG_ENTRIES {
            self.logs.drain(0..100);
        }
    }

    fn set_repo(&mut self, path: PathBuf) {
        self.current_branch = git::open(&path)
            .ok()
            .and_then(|repo| git::current_branch(&repo));
        self.repo_path = Some(path);
        self.commits.clear();
        self.clear_selection();
    }

    fn clear_selection(&mut self) {
        self.selected_index = None;
        self.new_message.clear();
    }

    /// Size of a full reload: keeps already loaded pages on refresh.
    fn reload_limit(&self) -> usize {
        self.commits.len().max(PAGE_SIZE)
    }

    /// Runs `task` on a background thread and routes its messages to the UI.
    fn spawn_task(
        &mut self,
        ctx: &egui::Context,
        task: impl FnOnce(PathBuf, Sender<BackgroundMessage>) + Send + 'static,
    ) -> bool {
        let Some(path) = self.repo_path.clone() else {
            self.add_log_typed("Repository not selected", LogLevel::Error);
            return false;
        };

        let (tx, rx) = mpsc::channel();
        self.message_receiver = Some(rx);

        let ctx = ctx.clone();
        std::thread::spawn(move || {
            task(path, tx);
            ctx.request_repaint();
        });
        true
    }

    pub fn load_commits_async(&mut self, ctx: &egui::Context) {
        let limit = self.reload_limit();
        if self.spawn_task(ctx, move |path, tx| {
            load_commits_task(&path, limit, 0, true, tx)
        }) {
            self.is_processing = true;
        }
    }

    pub fn load_more_commits_async(&mut self, ctx: &egui::Context) {
        if self.loading_more || self.is_processing {
            return;
        }
        let offset = self.commits.len();
        if self.spawn_task(ctx, move |path, tx| {
            load_commits_task(&path, PAGE_SIZE, offset, false, tx)
        }) {
            self.loading_more = true;
        }
    }

    pub fn rewrite_commit_async(
        &mut self,
        commit_hash: String,
        new_message: String,
        ctx: &egui::Context,
    ) {
        let scope = if self.modify_all_branches {
            self.add_log_typed(
                "Changes will be applied to all branches and tags",
                LogLevel::Warning,
            );
            RewriteScope::AllBranches
        } else {
            match &self.current_branch {
                Some(branch) => self.add_log(&format!(
                    "Changes will be applied only to branch {}",
                    branch
                )),
                None => self.add_log("Changes will be applied to detached HEAD"),
            }
            RewriteScope::CurrentBranch
        };

        let limit = self.reload_limit();
        if self.spawn_task(ctx, move |path, tx| {
            rewrite_commit_task(&path, &commit_hash, &new_message, scope, limit, tx)
        }) {
            self.start_operation();
        }
    }

    pub fn confirm_changes_async(&mut self, ctx: &egui::Context) {
        let limit = self.reload_limit();
        if self.spawn_task(ctx, move |path, tx| confirm_rewrite_task(&path, limit, tx)) {
            self.is_processing = true;
        }
    }

    pub fn rollback_changes_async(&mut self, ctx: &egui::Context) {
        self.add_log("Rolling back changes...");
        let limit = self.reload_limit();
        if self.spawn_task(ctx, move |path, tx| rollback_changes_task(&path, limit, tx)) {
            self.start_operation();
        }
    }

    fn start_operation(&mut self) {
        self.progress = 0.0;
        self.is_processing = true;
        self.show_progress = true;
    }

    fn finish_operation(&mut self) {
        self.is_processing = false;
        self.loading_more = false;
        self.show_progress = false;
    }

    pub fn process_background_messages(&mut self) {
        let Some(receiver) = self.message_receiver.as_ref() else {
            return;
        };
        let messages: Vec<_> = receiver.try_iter().collect();

        for msg in messages {
            match msg {
                BackgroundMessage::Log(log, level) => self.add_log_typed(&log, level),
                BackgroundMessage::Progress(p) => self.progress = p,
                BackgroundMessage::CommitsLoaded {
                    commits,
                    has_more,
                    reset,
                } => {
                    if reset {
                        self.commits = commits;
                        self.clear_selection();
                    } else {
                        self.commits.extend(commits);
                    }
                    self.has_more_commits = has_more;
                }
                BackgroundMessage::PreviewReady(data) => self.preview_data = Some(data),
                BackgroundMessage::Error(err) => {
                    self.add_log_typed(&err, LogLevel::Error);
                    self.finish_operation();
                }
                BackgroundMessage::Done => self.finish_operation(),
            }
        }
    }

    /// Opens the folder dialog without blocking the UI thread, so the window
    /// keeps responding to the compositor while the dialog is open.
    fn pick_repository(&mut self, ctx: &egui::Context) {
        if self.folder_receiver.is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.folder_receiver = Some(rx);

        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let folder = pollster::block_on(rfd::AsyncFileDialog::new().pick_folder());
            tx.send(folder.map(|handle| handle.path().to_path_buf()))
                .ok();
            ctx.request_repaint();
        });
    }

    fn process_picked_folder(&mut self, ctx: &egui::Context) {
        let Some(receiver) = &self.folder_receiver else {
            return;
        };
        let path = match receiver.try_recv() {
            Ok(Some(path)) => path,
            Ok(None) | Err(mpsc::TryRecvError::Disconnected) => {
                self.folder_receiver = None;
                return;
            }
            Err(mpsc::TryRecvError::Empty) => return,
        };
        self.folder_receiver = None;

        if !git::is_valid_repo(&path) {
            self.add_log_typed(
                "Selected directory is not a Git repository",
                LogLevel::Error,
            );
            return;
        }

        self.set_repo(path.clone());
        match &self.current_branch {
            Some(branch) => self.add_log(&format!(
                "Loading commits from {} (branch {})",
                theme::display_path(&path),
                branch
            )),
            None => self.add_log(&format!(
                "Loading commits from {}",
                theme::display_path(&path)
            )),
        }
        self.load_commits_async(ctx);
    }

    fn apply_change(&mut self, ctx: &egui::Context) {
        let Some(commit) = self
            .selected_index
            .and_then(|i| self.commits.get(i))
            .cloned()
        else {
            self.add_log_typed("Select a commit first", LogLevel::Warning);
            return;
        };

        self.add_log(&format!("Rewriting commit {}...", commit.short_hash));
        self.add_log(&format!("Old: {}", commit.summary));
        self.add_log(&format!(
            "New: {}",
            self.new_message.lines().next().unwrap_or("")
        ));

        let new_message = self.new_message.clone();
        self.rewrite_commit_async(commit.hash, new_message, ctx);
    }
}

impl eframe::App for CommitRewriterApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.initialized {
            self.initialized = true;
            if self.repo_path.is_some() {
                self.load_commits_async(ctx);
            }
        }

        self.process_background_messages();
        self.process_picked_folder(ctx);

        if self.is_processing || self.loading_more {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // the preview must be answered before anything else can change
        let busy =
            self.is_processing || self.preview_data.is_some() || self.folder_receiver.is_some();

        if let Some(preview) = &self.preview_data {
            let result = preview_modal::render_preview_modal(&ctx, preview, self.is_processing);

            if result.confirm_clicked {
                self.preview_data = None;
                self.confirm_changes_async(&ctx);
            } else if result.cancel_clicked {
                self.preview_data = None;
                self.rollback_changes_async(&ctx);
            }
        }

        egui::Panel::top("header")
            .frame(theme::panel_frame(ui))
            .show(ui, |ui| {
                let result = main_window::render_main_window(
                    ui,
                    self.repo_path.as_ref(),
                    busy,
                    &mut self.show_help,
                    &mut self.modify_all_branches,
                    self.current_branch.as_deref(),
                    self.show_progress,
                    self.progress,
                );

                if result.pick_folder_clicked {
                    self.pick_repository(&ctx);
                }

                if result.refresh_clicked {
                    self.add_log("Refreshing commits list...");
                    if let Some(path) = &self.repo_path {
                        self.current_branch = git::open(path)
                            .ok()
                            .and_then(|repo| git::current_branch(&repo));
                    }
                    self.load_commits_async(&ctx);
                }
            });

        egui::Panel::bottom("logs_panel")
            .frame(theme::panel_frame(ui))
            .resizable(true)
            .default_size(150.0)
            .min_size(90.0)
            .max_size(400.0)
            .show(ui, |ui| {
                if logs_panel::render_logs_panel(ui, &self.logs, &mut self.log_filter) {
                    self.logs.clear();
                }
            });

        egui::Panel::bottom("editor_panel")
            .frame(theme::panel_frame(ui))
            .show(ui, |ui| {
                let original = self
                    .selected_index
                    .and_then(|i| self.commits.get(i))
                    .map(|c| c.message.as_str());
                let result =
                    editor_panel::render_editor_panel(ui, &mut self.new_message, original, busy);

                if result.reset_clicked {
                    if let Some(original) = original {
                        self.new_message = original.trim_end().to_string();
                    }
                }
                if result.apply_clicked {
                    self.apply_change(&ctx);
                }
            });

        egui::CentralPanel::default_margins()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(theme::PANEL_MARGIN))
            .show(ui, |ui| {
                let result = commits_list::render_commits_list(
                    ui,
                    &self.commits,
                    self.selected_index,
                    &mut self.search_query,
                    busy,
                    self.loading_more,
                    self.has_more_commits,
                );

                if let Some(commit) = result.selected_index.and_then(|i| self.commits.get(i)) {
                    self.new_message = commit.message.trim_end().to_string();
                    self.selected_index = result.selected_index;
                }

                if result.load_more_clicked {
                    self.load_more_commits_async(&ctx);
                }
            });
    }
}

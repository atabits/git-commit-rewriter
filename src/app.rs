use crate::background::{
    load_commits_task, rewrite_commit_task, rollback_changes_task, BackgroundMessage,
};
use crate::git::{get_current_branch, GitRepositoryImpl};
use crate::models::{CommitInfo, LogEntry, LogFilter, LogLevel, PreviewData};
use crate::ui::{commits_list, editor_panel, logs_panel, main_window, preview_modal};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

pub struct CommitRewriterApp {
    pub repo_path: Option<PathBuf>,
    pub current_branch: Option<String>,
    pub modify_all_branches: bool,

    pub commits: Vec<CommitInfo>,
    pub selected_index: Option<usize>,
    pub commits_limit: usize,
    pub commits_loaded: usize,
    pub loading_more: bool,

    pub new_message: String,

    pub logs: Vec<LogEntry>,
    pub log_filter: LogFilter,

    pub progress: f32,
    pub show_progress: bool,
    pub is_processing: bool,

    pub message_receiver: Option<mpsc::Receiver<BackgroundMessage>>,

    pub show_help: bool,
    pub search_query: String,
    pub filtered_count: usize,

    pub show_preview_modal: bool,
    pub preview_data: Option<PreviewData>,
}

impl Default for CommitRewriterApp {
    fn default() -> Self {
        let current_dir = std::env::current_dir().ok();
        let mut app = Self {
            repo_path: None,
            current_branch: None,
            modify_all_branches: true,

            commits: Vec::new(),
            selected_index: None,
            commits_limit: 50,
            commits_loaded: 0,
            loading_more: false,

            new_message: String::new(),

            logs: Vec::new(),
            log_filter: LogFilter::All,

            progress: 0.0,
            show_progress: false,
            is_processing: false,

            message_receiver: None,

            show_help: false,
            search_query: String::new(),
            filtered_count: 0,

            show_preview_modal: false,
            preview_data: None,
        };

        // try to use current directory if it's a git repo
        if let Some(dir) = current_dir {
            if GitRepositoryImpl::is_valid_repo(&dir) {
                app.repo_path = Some(dir.clone());
                app.add_log(&format!("Initialized with directory: {}", dir.display()));
            } else {
                app.add_log("Select a Git repository to get started");
            }
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
        if self.logs.len() > 1000 {
            self.logs.drain(0..100);
        }
    }

    pub fn load_commits_async(&mut self, ctx: egui::Context) {
        self.load_commits_async_with_limit(ctx, self.commits_limit, 0, true);
    }

    pub fn load_more_commits_async(&mut self, ctx: egui::Context) {
        if self.loading_more || self.is_processing {
            return;
        }
        self.loading_more = true;
        self.load_commits_async_with_limit(ctx, 50, self.commits_loaded, false);
    }

    fn load_commits_async_with_limit(
        &mut self,
        ctx: egui::Context,
        limit: usize,
        offset: usize,
        reset: bool,
    ) {
        let path = match self.repo_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                self.add_log("❌ Repository not selected");
                return;
            }
        };

        if reset {
            self.add_log_typed("Requesting commits from repository...", LogLevel::Info);
            self.is_processing = true;
        } else {
            self.add_log_typed("Loading more commits...", LogLevel::Info);
        }

        let (tx, rx) = mpsc::channel();
        self.message_receiver = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            load_commits_task(path, limit, offset, reset, tx);
            ctx_clone.request_repaint();
        });

        ctx.request_repaint();
    }

    pub fn rewrite_commit_async(
        &mut self,
        commit_hash: String,
        new_message: String,
        ctx: egui::Context,
    ) {
        let path = match self.repo_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                self.add_log("❌ Repository not selected");
                return;
            }
        };

        // figure out current branch if we don't know it yet
        if self.current_branch.is_none() {
            self.current_branch = get_current_branch(&path);
        }

        let modify_all = self.modify_all_branches;
        let branch_name = self.current_branch.clone();

        self.add_log(&format!("Starting commit rewrite {}...", &commit_hash[..8]));
        if modify_all {
            self.add_log("⚠️ Changes will be applied to ALL branches");
        } else if let Some(ref branch) = branch_name {
            self.add_log(&format!(
                "📌 Changes will be applied only to branch: {}",
                branch
            ));
        }

        self.progress = 0.2;
        self.is_processing = true;
        self.show_progress = true;

        let (tx, rx) = mpsc::channel();
        self.message_receiver = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            rewrite_commit_task(path, commit_hash, new_message, modify_all, branch_name, tx);
            ctx_clone.request_repaint();
        });

        ctx.request_repaint();
    }

    pub fn rollback_changes_async(&mut self, ctx: egui::Context) {
        let path = match self.repo_path.as_ref() {
            Some(p) => p.clone(),
            None => {
                self.add_log("❌ Repository not selected");
                return;
            }
        };

        self.add_log("🔄 Rolling back changes...");
        self.is_processing = true;
        self.show_progress = true;
        self.progress = 0.1;

        let (tx, rx) = mpsc::channel();
        self.message_receiver = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            rollback_changes_task(path, tx);
            ctx_clone.request_repaint();
        });

        ctx.request_repaint();
    }

    pub fn process_background_messages(&mut self, ctx: &egui::Context) -> bool {
        let mut needs_repaint = false;
        let mut commit_rewritten = false;

        if let Some(receiver) = self.message_receiver.take() {
            let mut messages = Vec::new();
            loop {
                match receiver.try_recv() {
                    Ok(msg) => messages.push(msg),
                    Err(_) => break,
                }
            }

            for msg in messages {
                match msg {
                    BackgroundMessage::Log(log) => {
                        self.add_log(&log);
                        needs_repaint = true;
                    }
                    BackgroundMessage::LogTyped(log, level) => {
                        self.add_log_typed(&log, level);
                        needs_repaint = true;
                    }
                    BackgroundMessage::Progress(p) => {
                        self.progress = p;
                        needs_repaint = true;
                    }
                    BackgroundMessage::CommitsLoaded(commits) => {
                        if self.loading_more {
                            self.commits.extend(commits);
                            self.commits_loaded = self.commits.len();
                            self.loading_more = false;
                        } else {
                            self.commits = commits;
                            self.commits_loaded = self.commits.len();
                            self.commits_limit = self.commits_loaded;
                        }
                        self.filtered_count = self.commits.len();
                        needs_repaint = true;
                    }
                    BackgroundMessage::CommitRewritten => {
                        commit_rewritten = true;
                    }
                    BackgroundMessage::PreviewReady(data) => {
                        self.preview_data = Some(data);
                        self.show_preview_modal = true;
                        needs_repaint = true;
                    }
                    BackgroundMessage::Error(err) => {
                        self.add_log_typed(&format!("ERROR: {}", err), LogLevel::Error);
                        self.is_processing = false;
                        self.loading_more = false;
                        self.show_progress = false;
                        needs_repaint = true;
                    }
                    BackgroundMessage::Done => {
                        self.is_processing = false;
                        self.loading_more = false;
                        if self.progress >= 1.0 {
                            self.show_progress = false;
                        }
                        needs_repaint = true;
                    }
                }
            }

            if !commit_rewritten {
                self.message_receiver = Some(receiver);
            }
        }

        // reload commits after rewrite
        if commit_rewritten {
            self.load_commits_async(ctx.clone());
        }

        needs_repaint
    }
}

impl eframe::App for CommitRewriterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // initial load on startup
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            if self.repo_path.is_some() && self.commits.is_empty() && !self.is_processing {
                self.load_commits_async(ctx.clone());
            }
        });

        let needs_repaint = self.process_background_messages(ctx);

        if self.show_preview_modal {
            if let Some(preview) = self.preview_data.clone() {
                let result = preview_modal::render_preview_modal(ctx, &preview, self.is_processing);

                if result.confirm_clicked {
                    if self.repo_path.is_some() {
                        let (tx, rx) = mpsc::channel();
                        self.message_receiver = Some(rx);

                        std::thread::spawn(move || {
                            tx.send(BackgroundMessage::Log(
                                "Cleaning up temporary refs...".to_string(),
                            ))
                            .ok();
                            tx.send(BackgroundMessage::Log("✅ Changes confirmed!".to_string()))
                                .ok();
                            tx.send(BackgroundMessage::CommitRewritten).ok();
                            tx.send(BackgroundMessage::Done).ok();
                        });
                    }

                    self.show_preview_modal = false;
                    self.preview_data = None;
                }

                if result.cancel_clicked {
                    self.rollback_changes_async(ctx.clone());
                    self.show_preview_modal = false;
                    self.preview_data = None;
                }
            }
        }

        if needs_repaint {
            ctx.request_repaint();
        }

        egui::SidePanel::right("logs_panel")
            .resizable(true)
            .default_width(450.0)
            .min_width(300.0)
            .max_width(600.0)
            .show(ctx, |ui| {
                logs_panel::render_logs_panel(
                    ui,
                    &self.logs,
                    &mut self.log_filter,
                    self.show_progress,
                    self.progress,
                    self.is_processing,
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let main_result = main_window::render_main_window(
                ui,
                self.repo_path.as_ref(),
                &self.commits,
                self.is_processing,
                &mut self.show_help,
                &mut self.modify_all_branches,
                self.current_branch.as_ref(),
            );

            if main_result.pick_folder_clicked {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    if GitRepositoryImpl::is_valid_repo(&path) {
                        self.repo_path = Some(path.clone());
                        self.current_branch = get_current_branch(&path);
                        if let Some(ref branch) = self.current_branch {
                            self.add_log(&format!(
                                "✅ Loading commits from: {} (branch: {})",
                                path.display(),
                                branch
                            ));
                        } else {
                            self.add_log(&format!("✅ Loading commits from: {}", path.display()));
                        }
                        self.load_commits_async(ctx.clone());
                    } else {
                        self.add_log("❌ Selected directory is not a Git repository");
                    }
                }
            }

            if main_result.refresh_clicked {
                self.add_log("🔄 Refreshing commits list...");
                self.load_commits_async(ctx.clone());
            }

            ui.vertical(|ui| {
                let commits_result = commits_list::render_commits_list(
                    ui,
                    &self.commits,
                    self.selected_index,
                    &mut self.search_query,
                    self.is_processing,
                    self.loading_more,
                );

                if let Some((index, short_hash, message)) = commits_result.selected_commit {
                    self.selected_index = Some(index);
                    self.new_message = message.clone();
                    self.add_log(&format!("📝 Selected commit: {} - {}", short_hash, message));
                }

                if commits_result.load_more_clicked {
                    self.load_more_commits_async(ctx.clone());
                }

                if commits_result.search_changed {
                    self.filtered_count = self.commits.len();
                }

                ui.separator();

                let editor_result = editor_panel::render_editor_panel(
                    ui,
                    &mut self.new_message,
                    self.selected_index,
                    self.is_processing,
                );

                if editor_result.apply_clicked {
                    if let Some(index) = self.selected_index {
                        let commit = self.commits[index].clone();
                        let new_msg = self.new_message.clone();

                        self.add_log(&format!(
                            "🔄 Preparing to rewrite commit {}...",
                            commit.short_hash
                        ));
                        self.add_log(&format!("📝 Old: {}", commit.message));
                        self.add_log(&format!("📝 New: {}", new_msg));

                        self.rewrite_commit_async(commit.hash, new_msg, ctx.clone());
                    }
                }
            });
        });
    }
}

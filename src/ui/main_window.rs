use crate::ui::theme;
use eframe::egui::{self, RichText};

pub struct MainWindowResult {
    pub pick_folder_clicked: bool,
    pub refresh_clicked: bool,
}

/// Header with the repository, actions and rewrite scope.
#[allow(clippy::too_many_arguments)]
pub fn render_main_window(
    ui: &mut egui::Ui,
    repo_path: Option<&std::path::PathBuf>,
    is_processing: bool,
    show_help: &mut bool,
    modify_all_branches: &mut bool,
    current_branch: Option<&str>,
    show_progress: bool,
    progress: f32,
) -> MainWindowResult {
    let mut result = MainWindowResult {
        pick_folder_clicked: false,
        refresh_clicked: false,
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new("Git Commit Rewriter").heading().strong());
        ui.label(theme::muted(ui, format!("v{}", env!("CARGO_PKG_VERSION"))).small());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.toggle_value(show_help, "Help");
        });
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(!is_processing, egui::Button::new("📂 Open repository…"))
            .clicked()
        {
            result.pick_folder_clicked = true;
        }

        if ui
            .add_enabled(
                !is_processing && repo_path.is_some(),
                egui::Button::new("⟳ Refresh"),
            )
            .on_hover_text("Reload commits and the current branch")
            .clicked()
        {
            result.refresh_clicked = true;
        }

        ui.add_space(8.0);

        match repo_path {
            Some(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| theme::display_path(path));
                ui.label(RichText::new(name).strong())
                    .on_hover_text(theme::display_path(path));
                theme::chip(ui, current_branch.unwrap_or("detached HEAD"), theme::ACCENT);
            }
            None => {
                ui.label(theme::muted(ui, "No repository selected"));
            }
        }
    });

    if repo_path.is_some() {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(theme::muted(ui, "Rewrite"));
            ui.add_enabled_ui(!is_processing, |ui| {
                let branch_label = match current_branch {
                    Some(branch) => format!("current branch ({})", branch),
                    None => "detached HEAD".to_string(),
                };
                ui.radio_value(modify_all_branches, false, branch_label);
                ui.radio_value(modify_all_branches, true, "all branches and tags");
            });

            if *modify_all_branches {
                ui.label(
                    RichText::new("⚠ every branch and tag containing the commit changes")
                        .small()
                        .color(theme::WARNING),
                );
            }
        });
    }

    if *show_help {
        ui.add_space(6.0);
        theme::card(ui).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("How to use").strong());
            ui.label("1. Open a Git repository");
            ui.label("2. Select a commit and edit its message");
            ui.label("3. Apply, review the preview and keep or roll back the change");
            ui.add_space(4.0);
            ui.label(RichText::new("Good to know").strong());
            ui.label("• Rewriting changes the hashes of the commit and all its descendants");
            ui.label("• Push with git push --force-with-lease, don't rewrite shared branches");
            ui.label("• Original refs are kept until you confirm, rolling back restores them");
            ui.add_space(4.0);
            ui.label(theme::muted(ui, "Created by Amin Atabiev").small());
        });
    }

    if show_progress {
        ui.add_space(6.0);
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_height(6.0)
                .fill(theme::ACCENT)
                .animate(true),
        );
    }

    result
}

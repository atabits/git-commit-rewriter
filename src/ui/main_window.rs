use eframe::egui;

pub struct MainWindowResult {
    pub pick_folder_clicked: bool,
    pub refresh_clicked: bool,
}

pub fn render_main_window(
    ui: &mut egui::Ui,
    repo_path: Option<&std::path::PathBuf>,
    commits_count: usize,
    is_processing: bool,
    show_help: &mut bool,
    modify_all_branches: &mut bool,
    current_branch: Option<&str>,
) -> MainWindowResult {
    let mut result = MainWindowResult {
        pick_folder_clicked: false,
        refresh_clicked: false,
    };

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("🔧 Git Commit Rewriter")
                    .size(22.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❓").on_hover_text("Help").clicked() {
                    *show_help = !*show_help;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(
                    "Select a commit from the list, edit its message and apply the change",
                )
                .size(12.0)
                .color(egui::Color32::GRAY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("Created by Amin Atabiev")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(150, 200, 255)),
                );
            });
        });

        if *show_help {
            ui.add_space(3.0);
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(50, 60, 80))
                .corner_radius(5.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("💡 How to use:")
                            .strong()
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );
                    ui.label("1. Select a Git repository");
                    ui.label("2. Click on a commit in the list");
                    ui.label("3. Edit the message");
                    ui.label("4. Click 'Apply change'");
                    ui.label("5. Watch the logs on the right");

                    ui.add_space(3.0);
                    ui.label(
                        egui::RichText::new("⚠️ Important:")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 180, 50)),
                    );
                    ui.label(
                        "• Rewriting changes the hashes of the commit and all its descendants",
                    );
                    ui.label(
                        "• Push with git push --force-with-lease, don't rewrite shared branches",
                    );
                    ui.label("• Original refs are kept until you confirm, Cancel restores them");

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(3.0);
                    ui.label(
                        egui::RichText::new("ℹ️ About:")
                            .strong()
                            .color(egui::Color32::from_rgb(150, 200, 255)),
                    );
                    ui.label("Created by: Amin Atabiev");
                });
        }

        ui.add_space(3.0);

        ui.horizontal(|ui| {
            let btn = egui::Button::new(egui::RichText::new("📂 Select Repository").size(14.0))
                .min_size(egui::vec2(160.0, 32.0));

            if ui.add_enabled(!is_processing, btn).clicked() {
                result.pick_folder_clicked = true;
            }

            let refresh_btn = egui::Button::new(egui::RichText::new("🔄 Refresh").size(14.0))
                .min_size(egui::vec2(120.0, 32.0));

            if ui
                .add_enabled(!is_processing && repo_path.is_some(), refresh_btn)
                .clicked()
            {
                result.refresh_clicked = true;
            }

            if is_processing {
                ui.spinner();
                ui.label(
                    egui::RichText::new("Processing...")
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
            }
        });

        ui.add_space(2.0);

        if repo_path.is_some() {
            ui.horizontal(|ui| {
                ui.label("🌐 Modification mode:");

                ui.add_enabled(
                    !is_processing,
                    egui::Checkbox::new(
                        modify_all_branches,
                        egui::RichText::new("All branches and tags").size(12.0),
                    ),
                );

                if !*modify_all_branches {
                    if let Some(branch) = current_branch {
                        ui.label(
                            egui::RichText::new(format!("only {}", branch))
                                .monospace()
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 200, 255)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("only detached HEAD")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(255, 150, 50)),
                        );
                    }
                }

                if *modify_all_branches {
                    ui.label(
                        egui::RichText::new("⚠️")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(255, 180, 50)),
                    );
                    ui.label(
                        egui::RichText::new("Rewrites every branch and tag containing the commit")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(255, 180, 50))
                            .italics(),
                    );
                }
            });

            ui.add_space(2.0);
        }

        if let Some(path) = repo_path {
            ui.horizontal(|ui| {
                ui.label("📁");
                ui.label(
                    egui::RichText::new(format!("{}", path.display()))
                        .monospace()
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );

                if commits_count > 0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("📊 {} commits", commits_count))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 200, 100)),
                        );
                    });
                }
            });
        }

        ui.separator();
    });

    result
}

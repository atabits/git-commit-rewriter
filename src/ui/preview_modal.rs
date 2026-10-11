use crate::models::{PreviewData, RefChange};
use eframe::egui;

pub struct PreviewModalResult {
    pub confirm_clicked: bool,
    pub cancel_clicked: bool,
}

pub fn render_preview_modal(
    ctx: &egui::Context,
    preview_data: &PreviewData,
    is_processing: bool,
) -> PreviewModalResult {
    let mut result = PreviewModalResult {
        confirm_clicked: false,
        cancel_clicked: false,
    };

    egui::Window::new("🔍 Preview Changes")
        .collapsible(false)
        .resizable(true)
        .default_size([800.0, 600.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                match &preview_data.details {
                    Some(details) => {
                        ui.heading(
                            egui::RichText::new("Review changes before confirming")
                                .size(16.0)
                                .strong(),
                        );

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("📌 Commit:");
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} → {}",
                                    short(&details.old_hash),
                                    short(&details.new_hash)
                                ))
                                .monospace()
                                .size(12.0),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "({} commits rewritten)",
                                    details.rewritten_commits
                                ))
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                            );
                        });

                        ui.add_space(5.0);

                        message_block(
                            ui,
                            "❌ Old message:",
                            egui::Color32::from_rgb(255, 100, 100),
                            egui::Color32::from_rgb(40, 30, 30),
                            &details.old_message,
                        );

                        ui.add_space(5.0);

                        message_block(
                            ui,
                            "✅ New message:",
                            egui::Color32::from_rgb(100, 200, 100),
                            egui::Color32::from_rgb(30, 40, 30),
                            &details.new_message,
                        );
                    }
                    None => {
                        ui.heading(
                            egui::RichText::new("Unconfirmed rewrite found")
                                .size(16.0)
                                .strong(),
                        );
                        ui.add_space(5.0);
                        ui.label(
                            "A previous rewrite in this repository was neither confirmed nor \
                             rolled back. Confirm to keep it or cancel to restore the original refs.",
                        );
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                ui.label(egui::RichText::new("📋 Updated refs:").size(13.0).strong());
                ui.add_space(3.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for change in &preview_data.ref_changes {
                            ref_change_row(ui, change);
                        }
                    });

                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new(
                        "Cancel restores the refs above. After confirming, push with \
                         git push --force-with-lease.",
                    )
                    .size(11.0)
                    .color(egui::Color32::GRAY),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cancel_btn =
                            egui::Button::new(egui::RichText::new("❌ Cancel Changes").size(14.0))
                                .fill(egui::Color32::from_rgb(200, 80, 80));

                        if ui.add_enabled(!is_processing, cancel_btn).clicked() {
                            result.cancel_clicked = true;
                        }

                        ui.add_space(10.0);

                        let confirm_btn =
                            egui::Button::new(egui::RichText::new("✅ Confirm Changes").size(14.0))
                                .fill(egui::Color32::from_rgb(80, 200, 80));

                        if ui.add_enabled(!is_processing, confirm_btn).clicked() {
                            result.confirm_clicked = true;
                        }
                    });
                });
            });
        });

    result
}

fn message_block(
    ui: &mut egui::Ui,
    title: &str,
    title_color: egui::Color32,
    fill: egui::Color32,
    message: &str,
) {
    ui.label(
        egui::RichText::new(title)
            .size(13.0)
            .strong()
            .color(title_color),
    );
    egui::Frame::NONE
        .fill(fill)
        .corner_radius(3.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt(title)
                .max_height(120.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(message.trim_end())
                            .monospace()
                            .size(11.0),
                    );
                });
        });
}

fn ref_change_row(ui: &mut egui::Ui, change: &RefChange) {
    ui.label(
        egui::RichText::new(format!(
            "{}  {} → {}",
            change.name,
            short(&change.old_hash),
            short(&change.new_hash)
        ))
        .monospace()
        .size(10.0)
        .color(egui::Color32::GRAY),
    );
}

fn short(hash: &str) -> &str {
    hash.get(..8).unwrap_or(hash)
}

use crate::models::PreviewData;
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
                ui.heading(
                    egui::RichText::new("Review changes before confirming")
                        .size(16.0)
                        .strong(),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("📌 Commit hash:");
                    ui.label(
                        egui::RichText::new(&preview_data.commit_hash[..8])
                            .monospace()
                            .size(12.0),
                    );
                });

                ui.add_space(5.0);

                ui.label(
                    egui::RichText::new("❌ Old message:")
                        .size(13.0)
                        .strong()
                        .color(egui::Color32::from_rgb(255, 100, 100)),
                );
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(40, 30, 30))
                    .rounding(3.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&preview_data.old_message)
                                .monospace()
                                .size(11.0),
                        );
                    });

                ui.add_space(5.0);

                ui.label(
                    egui::RichText::new("✅ New message:")
                        .size(13.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 200, 100)),
                );
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(30, 40, 30))
                    .rounding(3.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&preview_data.new_message)
                                .monospace()
                                .size(11.0),
                        );
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                if !preview_data.affected_commits.is_empty() {
                    ui.label(egui::RichText::new("📋 Affected refs:").size(13.0).strong());
                    ui.add_space(3.0);

                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for ref_name in &preview_data.affected_commits {
                                ui.label(
                                    egui::RichText::new(ref_name)
                                        .monospace()
                                        .size(10.0)
                                        .color(egui::Color32::GRAY),
                                );
                            }
                        });

                    ui.add_space(5.0);
                }

                ui.label(
                    egui::RichText::new("📊 Commit history (last 10):")
                        .size(13.0)
                        .strong(),
                );
                ui.add_space(3.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(20, 20, 20))
                            .rounding(3.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&preview_data.diff_output)
                                        .monospace()
                                        .size(10.0),
                                );
                            });
                    });

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

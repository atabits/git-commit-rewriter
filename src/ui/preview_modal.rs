use crate::models::{PreviewData, RefChange};
use crate::ui::theme;
use eframe::egui::{self, RichText};

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

    let max_width = (ctx.content_rect().width() - 80.0).clamp(320.0, 720.0);

    egui::Modal::new(egui::Id::new("preview_modal")).show(ctx, |ui| {
        ui.set_width(max_width);

        match &preview_data.details {
            Some(details) => {
                ui.label(RichText::new("Review the rewrite").heading().strong());
                ui.horizontal(|ui| {
                    theme::chip(ui, theme::short_hash(&details.old_hash), theme::DANGER);
                    ui.label(RichText::new("→").monospace());
                    theme::chip(ui, theme::short_hash(&details.new_hash), theme::SUCCESS);
                    ui.label(theme::muted(
                        ui,
                        format!("{} commits rewritten", details.rewritten_commits),
                    ));
                });

                ui.add_space(8.0);
                message_block(ui, "Before", theme::DANGER, &details.old_message);
                ui.add_space(6.0);
                message_block(ui, "After", theme::SUCCESS, &details.new_message);
            }
            None => {
                ui.label(
                    RichText::new("Unconfirmed rewrite found")
                        .heading()
                        .strong(),
                );
                ui.label(
                    "A previous rewrite in this repository was neither kept nor rolled back. \
                     Decide what to do with it before making new changes.",
                );
            }
        }

        ui.add_space(10.0);
        ui.label(
            RichText::new(format!("Updated refs ({})", preview_data.ref_changes.len())).strong(),
        );
        ui.add_space(2.0);
        theme::card(ui).inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .id_salt("ref_changes")
                .max_height(120.0)
                .show(ui, |ui| {
                    for change in &preview_data.ref_changes {
                        ref_change_row(ui, change);
                    }
                });
        });

        ui.add_space(8.0);
        ui.label(
            theme::muted(
                ui,
                "Roll back restores the refs above. After keeping the changes, push them with \
                 git push --force-with-lease.",
            )
            .small(),
        );

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::primary_button(ui, "Keep changes", !is_processing).clicked() {
                    result.confirm_clicked = true;
                }
                if ui
                    .add_enabled(!is_processing, theme::danger_button("Roll back"))
                    .clicked()
                {
                    result.cancel_clicked = true;
                }
            });
        });
    });

    result
}

fn message_block(ui: &mut egui::Ui, title: &str, color: egui::Color32, message: &str) {
    ui.label(RichText::new(title).small().strong().color(color));
    egui::Frame::new()
        .fill(color.gamma_multiply(0.08))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.4)))
        .corner_radius(6)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .id_salt(title)
                .max_height(110.0)
                .show(ui, |ui| {
                    ui.add(egui::Label::new(RichText::new(message.trim_end()).monospace()).wrap());
                });
        });
}

fn ref_change_row(ui: &mut egui::Ui, change: &RefChange) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(&change.name).monospace());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                theme::muted(
                    ui,
                    format!(
                        "{} → {}",
                        theme::short_hash(&change.old_hash),
                        theme::short_hash(&change.new_hash)
                    ),
                )
                .monospace(),
            );
        });
    });
}

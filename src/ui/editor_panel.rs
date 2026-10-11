use crate::ui::theme;
use eframe::egui::{self, RichText};

/// Recommended maximum length of the first line.
const SUBJECT_LIMIT: usize = 72;
const EDITOR_MAX_HEIGHT: f32 = 180.0;

pub struct EditorPanelResult {
    pub apply_clicked: bool,
    pub reset_clicked: bool,
}

/// Message editor. `original` is the current message of the selected commit.
pub fn render_editor_panel(
    ui: &mut egui::Ui,
    new_message: &mut String,
    original: Option<&str>,
    is_processing: bool,
) -> EditorPanelResult {
    let mut result = EditorPanelResult {
        apply_clicked: false,
        reset_clicked: false,
    };

    let changed = original.is_some_and(|o| o.trim_end() != new_message.trim_end());

    ui.horizontal(|ui| {
        ui.label(theme::section_title("Message"));

        if original.is_some() {
            let len = new_message.lines().next().unwrap_or("").chars().count();
            let color = if len > SUBJECT_LIMIT {
                theme::WARNING
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(
                RichText::new(format!("subject {}/{}", len, SUBJECT_LIMIT))
                    .small()
                    .color(color),
            )
            .on_hover_text("Keep the first line within 72 characters");
        }
    });

    ui.add_space(4.0);

    // long messages scroll instead of pushing the commit list away
    egui::ScrollArea::vertical()
        .id_salt("message_editor")
        .max_height(EDITOR_MAX_HEIGHT)
        .show(ui, |ui| {
            ui.add_enabled(
                original.is_some() && !is_processing,
                egui::TextEdit::multiline(new_message)
                    .desired_width(f32::INFINITY)
                    .desired_rows(6)
                    .hint_text("Select a commit to edit its message")
                    .font(egui::TextStyle::Monospace)
                    .margin(egui::Margin::same(8)),
            );
        });

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        if changed {
            ui.label(theme::muted(ui, "Unsaved changes").small());
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::primary_button(
                ui,
                "Apply change",
                changed && !is_processing && !new_message.trim().is_empty(),
            )
            .on_disabled_hover_text("Edit the message of a selected commit first")
            .clicked()
            {
                result.apply_clicked = true;
            }

            if ui
                .add_enabled(changed && !is_processing, egui::Button::new("Reset"))
                .on_hover_text("Restore the current message")
                .clicked()
            {
                result.reset_clicked = true;
            }
        });
    });

    result
}

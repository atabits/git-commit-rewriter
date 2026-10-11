use crate::models::{LogEntry, LogFilter};
use crate::ui::theme;
use eframe::egui::{self, RichText};

/// Activity log. Returns `true` when Clear was clicked.
pub fn render_logs_panel(ui: &mut egui::Ui, logs: &[LogEntry], log_filter: &mut LogFilter) -> bool {
    let mut clear_clicked = false;

    ui.horizontal(|ui| {
        ui.label(theme::section_title("Activity"));
        ui.label(theme::muted(ui, logs.len().to_string()));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(!logs.is_empty(), egui::Button::new("Clear").small())
                .clicked()
            {
                clear_clicked = true;
            }

            egui::ComboBox::from_id_salt("log_filter")
                .selected_text(log_filter.name())
                .width(110.0)
                .show_ui(ui, |ui| {
                    for filter in [LogFilter::All, LogFilter::Important, LogFilter::ErrorsOnly] {
                        ui.selectable_value(log_filter, filter, filter.name());
                    }
                });
        });
    });

    ui.add_space(4.0);

    theme::card(ui).inner_margin(6).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if logs.is_empty() {
                    ui.label(theme::muted(ui, "Nothing here yet").italics());
                }
                for entry in logs.iter().filter(|e| log_filter.should_show(&e.level)) {
                    render_log_entry(ui, entry);
                }
            });
    });

    clear_clicked
}

fn render_log_entry(ui: &mut egui::Ui, entry: &LogEntry) {
    let (r, g, b) = entry.level.color();
    let color = egui::Color32::from_rgb(r, g, b);

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new(entry.level.icon()).color(color).monospace());
        ui.label(theme::muted(ui, &entry.timestamp).monospace().size(12.0));
        ui.add(egui::Label::new(RichText::new(&entry.message).size(13.0).color(color)).wrap());
    });
}

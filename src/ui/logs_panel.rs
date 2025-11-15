use crate::models::{LogEntry, LogFilter};
use eframe::egui;

pub fn render_logs_panel(
    ui: &mut egui::Ui,
    logs: &[LogEntry],
    log_filter: &mut LogFilter,
    show_progress: bool,
    progress: f32,
    is_processing: bool,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("📝 Logs").size(18.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear").clicked() && !is_processing {}

                egui::ComboBox::from_id_source("log_filter")
                    .selected_text(log_filter.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(log_filter, LogFilter::All, LogFilter::All.name());
                        ui.selectable_value(
                            log_filter,
                            LogFilter::Important,
                            LogFilter::Important.name(),
                        );
                        ui.selectable_value(
                            log_filter,
                            LogFilter::ErrorsOnly,
                            LogFilter::ErrorsOnly.name(),
                        );
                    });
            });
        });

        ui.add_space(2.0);

        ui.label(
            egui::RichText::new(format!("📊 Entries: {}", logs.len()))
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.separator();

        if show_progress {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("⏳");
                    ui.label(
                        egui::RichText::new("Executing operation...")
                            .color(egui::Color32::from_rgb(100, 150, 255)),
                    );
                });

                let progress_bar = egui::ProgressBar::new(progress)
                    .show_percentage()
                    .animate(true);
                ui.add(progress_bar);
            });
            ui.add_space(3.0);
            ui.separator();
        }

        let remaining_height = ui.available_height();
        egui::ScrollArea::vertical()
            .stick_to_bottom(false)
            .auto_shrink([false, true])
            .max_height(remaining_height.min(800.0))
            .show(ui, |ui| {
                if logs.is_empty() {
                    ui.add_space(ui.available_height() * 0.1);
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("📝")
                                    .size(48.0)
                                    .color(egui::Color32::from_rgb(100, 100, 120)),
                            );
                            ui.label(
                                egui::RichText::new("Logs will appear here")
                                    .size(13.0)
                                    .color(egui::Color32::GRAY)
                                    .italics(),
                            );
                        });
                    });
                } else {
                    for entry in logs {
                        if log_filter.should_show(&entry.level) {
                            render_log_entry(ui, entry);
                        }
                    }
                }
            });
    });
}

fn render_log_entry(ui: &mut egui::Ui, entry: &LogEntry) {
    let icon = entry.level.icon();
    let (r, g, b) = entry.level.color();
    let color = egui::Color32::from_rgb(r, g, b);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(12.0));
        ui.label(
            egui::RichText::new(&entry.timestamp)
                .monospace()
                .size(10.0)
                .color(egui::Color32::GRAY),
        );
        ui.label(
            egui::RichText::new(&entry.message)
                .monospace()
                .size(10.5)
                .color(color),
        );
    });
}

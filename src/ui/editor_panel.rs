use eframe::egui;

pub struct EditorPanelResult {
    pub apply_clicked: bool,
}

pub fn render_editor_panel(
    ui: &mut egui::Ui,
    new_message: &mut String,
    selected_index: Option<usize>,
    is_processing: bool,
) -> EditorPanelResult {
    let mut result = EditorPanelResult {
        apply_clicked: false,
    };

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("✏️ New Commit Message")
                    .size(14.0)
                    .strong(),
            );

            if !new_message.trim().is_empty() {
                let len = new_message.lines().next().unwrap_or("").len();
                let color = if len > 72 {
                    egui::Color32::from_rgb(255, 180, 50)
                } else {
                    egui::Color32::from_rgb(100, 200, 100)
                };

                ui.label(
                    egui::RichText::new(format!("{} characters", len))
                        .size(11.0)
                        .color(color),
                );

                if len > 72 {
                    ui.label(
                        egui::RichText::new("⚠️ Recommended ≤72")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(255, 180, 50)),
                    );
                }
            }
        });

        ui.add_space(2.0);

        let text_edit = egui::TextEdit::multiline(new_message)
            .desired_width(f32::INFINITY)
            .desired_rows(4)
            .hint_text("Enter new commit message...")
            .font(egui::TextStyle::Monospace);

        ui.add_enabled(!is_processing, text_edit);

        ui.add_space(3.0);

        ui.horizontal(|ui| {
            let apply_btn = egui::Button::new(egui::RichText::new("✅ Apply Change").size(14.0))
                .min_size(egui::vec2(180.0, 36.0));

            if ui
                .add_enabled(
                    !is_processing && selected_index.is_some() && !new_message.trim().is_empty(),
                    apply_btn,
                )
                .clicked()
            {
                result.apply_clicked = true;
            }

            if selected_index.is_none() {
                ui.label(
                    egui::RichText::new("← Select a commit from the list")
                        .size(11.0)
                        .color(egui::Color32::GRAY)
                        .italics(),
                );
            }
        });
    });

    result
}

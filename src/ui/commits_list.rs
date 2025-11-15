use crate::models::CommitInfo;
use eframe::egui;

pub struct CommitsListResult {
    pub selected_commit: Option<(usize, String, String)>, // (index, short_hash, message)
    pub load_more_clicked: bool,
    pub search_changed: bool,
}

pub fn render_commits_list(
    ui: &mut egui::Ui,
    commits: &[CommitInfo],
    selected_index: Option<usize>,
    search_query: &mut String,
    is_processing: bool,
    loading_more: bool,
    has_more_commits: bool,
) -> CommitsListResult {
    let mut result = CommitsListResult {
        selected_commit: None,
        load_more_clicked: false,
        search_changed: false,
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("📋 Commits List").size(14.0).strong());

        if !commits.is_empty() {
            let filtered = filter_commits(commits, search_query);
            let total = commits.len();
            let filtered_count = filtered.len();

            if !search_query.trim().is_empty() && filtered_count != total {
                ui.label(
                    egui::RichText::new(format!("({} of {})", filtered_count, total))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 150, 255)),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!("({})", total))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            }
        }
    });

    ui.add_space(2.0);

    if !commits.is_empty() {
        ui.horizontal(|ui| {
            let search_response = ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("🔍 Search by hash, message, author...")
                    .desired_width(f32::INFINITY),
            );

            if search_response.changed() {
                result.search_changed = true;
            }

            if !search_query.is_empty() {
                if ui.small_button("✖").on_hover_text("Clear search").clicked() {
                    search_query.clear();
                    result.search_changed = true;
                }
            }
        });
        ui.add_space(2.0);
    }

    let available_height = ui.available_height() - 150.0;
    egui::ScrollArea::vertical()
        .max_height(available_height.max(100.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if commits.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Select a Git repository\nto view commits")
                            .color(egui::Color32::GRAY)
                            .italics()
                            .size(12.0),
                    );
                });
            } else {
                let filtered = filter_commits(commits, search_query);

                if filtered.is_empty() && !search_query.trim().is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("🔍")
                                    .size(32.0)
                                    .color(egui::Color32::GRAY),
                            );
                            ui.label(
                                egui::RichText::new("Nothing found")
                                    .size(13.0)
                                    .color(egui::Color32::GRAY),
                            );
                            ui.label(
                                egui::RichText::new(format!("For query: \"{}\"", search_query))
                                    .size(11.0)
                                    .color(egui::Color32::DARK_GRAY)
                                    .italics(),
                            );
                        });
                    });
                } else {
                    for (original_index, commit) in filtered.iter() {
                        let i = *original_index;
                        let is_selected = selected_index == Some(i);

                        let full_text = format!(
                            "{} │ {} │ {}",
                            commit.short_hash, commit.date, commit.message
                        );

                        // Create selectable with highlighted matches
                        let response = if !search_query.trim().is_empty() {
                            render_selectable_with_highlight(
                                ui,
                                is_selected,
                                &full_text,
                                search_query,
                            )
                        } else {
                            ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&full_text).monospace().size(11.0),
                            )
                        };

                        if response.clicked() && !is_processing {
                            result.selected_commit =
                                Some((i, commit.short_hash.clone(), commit.message.clone()));
                        }

                        response.on_hover_text(format!(
                            "Hash: {}\nAuthor: {}\nDate: {}\nMessage: {}",
                            commit.hash, commit.author, commit.date, commit.message
                        ));
                    }

                    // Only show "Load more" button if there are potentially more commits
                    if has_more_commits {
                        ui.add_space(3.0);
                        ui.separator();
                        ui.add_space(3.0);

                        ui.centered_and_justified(|ui| {
                            if loading_more {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        egui::RichText::new("Loading...")
                                            .size(11.0)
                                            .color(egui::Color32::GRAY),
                                    );
                                });
                            } else {
                                let load_more_btn = egui::Button::new(
                                    egui::RichText::new("⬇ Load 50 more commits").size(12.0),
                                );

                                if ui
                                    .add_enabled(!is_processing, load_more_btn)
                                    .on_hover_text("Load next 50 commits")
                                    .clicked()
                                {
                                    result.load_more_clicked = true;
                                }
                            }
                        });
                    }
                }
            }
        });

    result
}

fn filter_commits<'a>(commits: &'a [CommitInfo], query: &str) -> Vec<(usize, &'a CommitInfo)> {
    if query.trim().is_empty() {
        commits.iter().enumerate().collect()
    } else {
        let query_lower = query.to_lowercase();
        commits
            .iter()
            .enumerate()
            .filter(|(_, commit)| {
                commit.message.to_lowercase().contains(&query_lower)
                    || commit.hash.to_lowercase().contains(&query_lower)
                    || commit.author.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

fn render_selectable_with_highlight(
    ui: &mut egui::Ui,
    is_selected: bool,
    text: &str,
    query: &str,
) -> egui::Response {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    // Find all match positions
    let mut matches = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = text_lower[search_start..].find(&query_lower) {
        let actual_pos = search_start + pos;
        matches.push((actual_pos, actual_pos + query.len()));
        search_start = actual_pos + 1;
    }

    if matches.is_empty() {
        return ui.selectable_label(
            is_selected,
            egui::RichText::new(text).monospace().size(11.0),
        );
    }

    // Create a custom selectable widget with highlighted text
    let button_padding = ui.style().spacing.button_padding;
    let text_height = ui.text_style_height(&egui::TextStyle::Monospace);

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), text_height + button_padding.y * 2.0),
        egui::Sense::click(),
    );

    if is_selected {
        let selection_color = ui.style().visuals.selection.bg_fill;
        ui.painter().rect_filled(rect, 0.0, selection_color);
    }

    if response.hovered() {
        let hover_color = ui.style().visuals.widgets.hovered.bg_fill;
        ui.painter().rect_filled(rect, 0.0, hover_color);
    }

    // Render text with highlights
    let mut last_end = 0;
    let text_pos = rect.min + button_padding;
    let mut current_x = text_pos.x;

    for (start, end) in matches {
        // Render text before match
        if start > last_end {
            let segment = &text[last_end..start];
            let text_color = ui.style().visuals.text_color();
            ui.painter().text(
                egui::pos2(current_x, text_pos.y),
                egui::Align2::LEFT_TOP,
                segment,
                egui::FontId::monospace(11.0),
                text_color,
            );
            current_x += ui.fonts(|f| {
                f.layout_no_wrap(
                    segment.to_string(),
                    egui::FontId::monospace(11.0),
                    text_color,
                )
                .size()
                .x
            });
        }

        // Render highlighted match
        let segment = &text[start..end];
        let highlight_rect = egui::Rect::from_min_size(
            egui::pos2(current_x, text_pos.y),
            egui::vec2(
                ui.fonts(|f| {
                    f.layout_no_wrap(
                        segment.to_string(),
                        egui::FontId::monospace(11.0),
                        egui::Color32::BLACK,
                    )
                    .size()
                    .x
                }),
                ui.text_style_height(&egui::TextStyle::Monospace),
            ),
        );
        ui.painter()
            .rect_filled(highlight_rect, 0.0, egui::Color32::from_rgb(255, 255, 0));
        ui.painter().text(
            egui::pos2(current_x, text_pos.y),
            egui::Align2::LEFT_TOP,
            segment,
            egui::FontId::monospace(11.0),
            egui::Color32::BLACK,
        );
        current_x += highlight_rect.width();

        last_end = end;
    }

    // Render remaining text
    if last_end < text.len() {
        let segment = &text[last_end..];
        let text_color = ui.style().visuals.text_color();
        ui.painter().text(
            egui::pos2(current_x, text_pos.y),
            egui::Align2::LEFT_TOP,
            segment,
            egui::FontId::monospace(11.0),
            text_color,
        );
    }

    response
}

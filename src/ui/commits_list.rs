use crate::models::CommitInfo;
use eframe::egui;

pub struct CommitsListResult {
    pub selected_index: Option<usize>,
    pub load_more_clicked: bool,
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
        selected_index: None,
        load_more_clicked: false,
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
            ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("🔍 Search by hash, message, author...")
                    .desired_width(f32::INFINITY),
            );

            if !search_query.is_empty()
                && ui.small_button("✖").on_hover_text("Clear search").clicked()
            {
                search_query.clear();
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
                            commit.short_hash, commit.date, commit.summary
                        );

                        let response = ui.selectable_label(
                            is_selected,
                            highlighted_text(ui, &full_text, search_query.trim()),
                        );

                        if response.clicked() && !is_processing {
                            result.selected_index = Some(i);
                        }

                        response.on_hover_text(format!(
                            "Hash: {}\nAuthor: {}\nDate: {}\n\n{}",
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
    let query = query.trim();
    commits
        .iter()
        .enumerate()
        .filter(|(_, commit)| {
            query.is_empty()
                || !find_matches(&commit.message, query).is_empty()
                || !find_matches(&commit.hash, query).is_empty()
                || !find_matches(&commit.author, query).is_empty()
        })
        .collect()
}

/// Byte ranges of case-insensitive, non-overlapping matches of `query` in `text`.
///
/// Works on chars, so ranges always fall on char boundaries of `text` even when
/// lowercasing changes the byte length of a character.
fn find_matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    let query: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    if query.is_empty() {
        return Vec::new();
    }

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut matches = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let mut matched = 0;
        let mut j = i;
        while j < chars.len() && matched < query.len() {
            let lower: Vec<char> = chars[j].1.to_lowercase().collect();
            if query[matched..].starts_with(&lower) {
                matched += lower.len();
                j += 1;
            } else {
                break;
            }
        }

        if matched == query.len() {
            let end = chars.get(j).map_or(text.len(), |(pos, _)| *pos);
            matches.push((chars[i].0, end));
            i = j;
        } else {
            i += 1;
        }
    }

    matches
}

fn highlighted_text(ui: &egui::Ui, text: &str, query: &str) -> egui::text::LayoutJob {
    let font_id = egui::FontId::monospace(11.0);
    let normal = egui::TextFormat {
        font_id: font_id.clone(),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    let highlight = egui::TextFormat {
        font_id,
        color: egui::Color32::BLACK,
        background: egui::Color32::from_rgb(255, 255, 0),
        ..Default::default()
    };

    let mut job = egui::text::LayoutJob::default();
    let mut last_end = 0;
    for (start, end) in find_matches(text, query) {
        job.append(&text[last_end..start], 0.0, normal.clone());
        job.append(&text[start..end], 0.0, highlight.clone());
        last_end = end;
    }
    job.append(&text[last_end..], 0.0, normal);
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matches_case_insensitive() {
        assert_eq!(
            find_matches("Fix Bug in fix", "fix"),
            vec![(0, 3), (11, 14)]
        );
        assert!(find_matches("abc", "x").is_empty());
        assert!(find_matches("abc", "").is_empty());
    }

    #[test]
    fn test_find_matches_unicode_boundaries() {
        // 'İ' lowercases to two chars, which used to shift byte offsets
        let text = "İstanbul fix";
        let matches = find_matches(text, "fix");
        assert_eq!(matches.len(), 1);
        let (start, end) = matches[0];
        assert_eq!(&text[start..end], "fix");

        let text = "Исправить ошибку";
        let (start, end) = find_matches(text, "ОШИБ")[0];
        assert_eq!(&text[start..end], "ошиб");
    }
}

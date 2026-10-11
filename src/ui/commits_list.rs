use crate::models::CommitInfo;
use crate::ui::theme;
use eframe::egui::{self, text::LayoutJob, Color32, FontId, Sense, TextFormat};

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

    let query = search_query.trim().to_string();
    let filtered = filter_commits(commits, &query);

    ui.horizontal(|ui| {
        ui.label(theme::section_title("Commits"));
        let count = if query.is_empty() {
            commits.len().to_string()
        } else {
            format!("{} of {}", filtered.len(), commits.len())
        };
        ui.label(theme::muted(ui, count));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !search_query.is_empty()
                && ui.small_button("✖").on_hover_text("Clear search").clicked()
            {
                search_query.clear();
            }
            ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("🔍 Search message, hash, author")
                    .desired_width(260.0),
            );
        });
    });

    ui.add_space(4.0);

    theme::card(ui).inner_margin(4).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if commits.is_empty() {
                    empty_state(
                        ui,
                        "No commits to show",
                        "Open a Git repository to get started",
                    );
                    return;
                }
                if filtered.is_empty() {
                    empty_state(
                        ui,
                        "Nothing found",
                        &format!("No commits match \"{}\"", query),
                    );
                    return;
                }

                for (index, commit) in &filtered {
                    let response = commit_row(ui, commit, selected_index == Some(*index), &query);
                    if response.clicked() && !is_processing {
                        result.selected_index = Some(*index);
                    }
                }

                if has_more_commits {
                    ui.add_space(6.0);
                    ui.vertical_centered(|ui| {
                        if loading_more {
                            ui.spinner();
                        } else if ui
                            .add_enabled(!is_processing, egui::Button::new("Load more commits"))
                            .clicked()
                        {
                            result.load_more_clicked = true;
                        }
                    });
                    ui.add_space(6.0);
                }
            });
    });

    result
}

fn commit_row(
    ui: &mut egui::Ui,
    commit: &CommitInfo,
    selected: bool,
    query: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), theme::ROW_HEIGHT),
        Sense::click(),
    );

    let visuals = ui.visuals().clone();
    let painter = ui.painter_at(rect);

    if selected {
        painter.rect_filled(rect, 6.0, visuals.selection.bg_fill);
        painter.rect_filled(
            egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 6.0),
                egui::vec2(3.0, rect.height() - 12.0),
            ),
            2.0,
            theme::ACCENT,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 6.0, visuals.faint_bg_color);
    }
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(1.0),
            6.0,
            egui::Stroke::new(1.5, theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    }

    let text_color = visuals.text_color();
    let weak_color = visuals.weak_text_color();
    let padding = 12.0;
    let center_y = rect.center().y;

    // hash on the left
    let hash_job = highlighted_text(
        &commit.short_hash,
        query,
        FontId::monospace(13.0),
        theme::ACCENT,
        None,
    );
    let hash = painter.layout_job(hash_job);
    let hash_pos = egui::pos2(rect.left() + padding, center_y - hash.size().y / 2.0);
    let summary_left = hash_pos.x + hash.size().x + 14.0;
    painter.galley(hash_pos, hash, text_color);

    // author and date on the right
    let meta_text = format!("{} · {}", commit.author, commit.date);
    let meta_job = highlighted_text(
        &meta_text,
        query,
        FontId::proportional(12.5),
        weak_color,
        None,
    );
    let meta = painter.layout_job(meta_job);
    let meta_pos = egui::pos2(
        rect.right() - padding - meta.size().x,
        center_y - meta.size().y / 2.0,
    );
    let summary_right = meta_pos.x - 16.0;
    painter.galley(meta_pos, meta, weak_color);

    // summary in between, truncated to one line
    let width = (summary_right - summary_left).max(40.0);
    let summary_job = highlighted_text(
        &commit.summary,
        query,
        FontId::proportional(14.0),
        text_color,
        Some(width),
    );
    let summary = painter.layout_job(summary_job);
    painter.galley(
        egui::pos2(summary_left, center_y - summary.size().y / 2.0),
        summary,
        text_color,
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!(
            "{}\n{} · {}\n\n{}",
            commit.hash,
            commit.author,
            commit.date,
            commit.message.trim_end()
        ))
}

fn empty_state(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.add_space(ui.available_height() * 0.3);
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(title).size(16.0).strong());
        ui.label(theme::muted(ui, subtitle));
    });
}

fn filter_commits<'a>(commits: &'a [CommitInfo], query: &str) -> Vec<(usize, &'a CommitInfo)> {
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

/// Text with search matches highlighted, optionally truncated to one line of `max_width`.
fn highlighted_text(
    text: &str,
    query: &str,
    font_id: FontId,
    color: Color32,
    max_width: Option<f32>,
) -> LayoutJob {
    let normal = TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };
    let highlight = TextFormat {
        font_id,
        color: Color32::BLACK,
        background: Color32::from_rgb(255, 214, 102),
        ..Default::default()
    };

    let mut job = LayoutJob::default();
    let mut last_end = 0;
    for (start, end) in find_matches(text, query) {
        job.append(&text[last_end..start], 0.0, normal.clone());
        job.append(&text[start..end], 0.0, highlight.clone());
        last_end = end;
    }
    job.append(&text[last_end..], 0.0, normal);

    if let Some(max_width) = max_width {
        job.wrap = egui::text::TextWrapping {
            max_width,
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('…'),
        };
    }
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

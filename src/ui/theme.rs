//! Shared look: spacing, colors and small widget helpers.

use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, TextStyle, Theme};

/// Orange from the app icon.
pub const ACCENT: Color32 = Color32::from_rgb(240, 136, 62);
pub const DANGER: Color32 = Color32::from_rgb(220, 90, 90);
pub const SUCCESS: Color32 = Color32::from_rgb(80, 180, 110);
pub const WARNING: Color32 = Color32::from_rgb(230, 170, 50);

pub const ROW_HEIGHT: f32 = 30.0;
pub const PANEL_MARGIN: i8 = 12;

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.text_edit_width = 240.0;

        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(18.0)),
            (TextStyle::Body, FontId::proportional(14.0)),
            (TextStyle::Button, FontId::proportional(14.0)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
            (TextStyle::Small, FontId::proportional(11.5)),
        ]
        .into();

        let radius = CornerRadius::same(6);
        let widgets = &mut style.visuals.widgets;
        for w in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            w.corner_radius = radius;
        }
        style.visuals.window_corner_radius = CornerRadius::same(10);
        style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        style.visuals.hyperlink_color = ACCENT;
    });

    ctx.style_mut_of(Theme::Dark, |style| {
        let v = &mut style.visuals;
        v.panel_fill = Color32::from_rgb(30, 31, 34);
        v.window_fill = Color32::from_rgb(38, 40, 44);
        v.extreme_bg_color = Color32::from_rgb(22, 23, 25);
        v.faint_bg_color = Color32::from_rgb(36, 37, 41);
        v.selection.bg_fill = Color32::from_rgb(92, 58, 34);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 52, 57));
    });

    ctx.style_mut_of(Theme::Light, |style| {
        let v = &mut style.visuals;
        v.panel_fill = Color32::from_rgb(246, 246, 248);
        v.window_fill = Color32::WHITE;
        v.extreme_bg_color = Color32::WHITE;
        v.faint_bg_color = Color32::from_rgb(236, 237, 240);
        v.selection.bg_fill = Color32::from_rgb(253, 222, 199);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(222, 223, 228));
    });
}

/// Frame for top and bottom panels.
pub fn panel_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::side_top_panel(ui.style())
        .inner_margin(egui::Margin::symmetric(PANEL_MARGIN, 10))
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
}

/// Bordered box used to group content inside a panel.
pub fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(10)
        .corner_radius(8)
}

pub fn section_title(text: &str) -> RichText {
    RichText::new(text).strong().size(15.0)
}

pub fn muted(ui: &egui::Ui, text: impl Into<String>) -> RichText {
    RichText::new(text).color(ui.visuals().weak_text_color())
}

/// Main action of a section: filled with the accent color while enabled.
pub fn primary_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let button = if enabled {
        egui::Button::new(
            RichText::new(text)
                .strong()
                .color(Color32::from_rgb(30, 20, 12)),
        )
        .fill(ACCENT)
    } else {
        egui::Button::new(RichText::new(text).strong())
    };
    ui.add_enabled(enabled, button.min_size(egui::vec2(0.0, 32.0)))
}

/// Outlined button for destructive actions.
pub fn danger_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).color(DANGER))
        .stroke(Stroke::new(1.0, DANGER))
        .fill(Color32::TRANSPARENT)
        .min_size(egui::vec2(0.0, 32.0))
}

/// Small rounded label, e.g. for the branch name.
pub fn chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.18))
        .corner_radius(10)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(12.0).color(color));
        });
}

/// Path for display, with the home directory shown as `~`.
pub fn display_path(path: &std::path::Path) -> String {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Some(rest) = home.and_then(|home| path.strip_prefix(home).ok().map(|r| r.to_path_buf()))
    {
        return std::path::Path::new("~").join(rest).display().to_string();
    }
    path.display().to_string()
}

pub fn short_hash(hash: &str) -> &str {
    hash.get(..8).unwrap_or(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_hash() {
        assert_eq!(short_hash("0123456789abcdef"), "01234567");
        assert_eq!(short_hash("abc"), "abc");
    }

    #[test]
    fn test_display_path_outside_home() {
        assert_eq!(display_path(std::path::Path::new("/opt/repo")), "/opt/repo");
    }
}

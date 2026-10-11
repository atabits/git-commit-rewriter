#![cfg_attr(windows, windows_subsystem = "windows")]

use commit_rewriter::app::CommitRewriterApp;
use commit_rewriter::ui::theme;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 760.0])
            .with_min_inner_size([640.0, 520.0])
            // matches commit-rewriter.desktop, so Wayland shows its icon
            .with_app_id("commit-rewriter")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon/icon-256.png"))
                    .expect("bundled icon is a valid PNG"),
            )
            .with_title("Git Commit Rewriter"),
        ..Default::default()
    };

    eframe::run_native(
        "Git Commit Rewriter",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(CommitRewriterApp::default()))
        }),
    )
}

#![cfg_attr(windows, windows_subsystem = "windows")]

use commit_rewriter::app::CommitRewriterApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Git Commit Rewriter"),
        ..Default::default()
    };

    eframe::run_native(
        "Git Commit Rewriter",
        options,
        Box::new(|_cc| Ok(Box::new(CommitRewriterApp::default()))),
    )
}

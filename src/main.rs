mod app;
mod config;
mod ui;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub use app::{YtDlpApp, FormatInfo, short_codec};
use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(448.0, 256.0)),
        decorated: true,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Box::new(YtDlpApp::default())),
    ) {
        eprintln!("Error: {}", e);
    }
}

impl eframe::App for YtDlpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();
        ui::render_ui(self, ctx);
        if self.is_busy() {
            ctx.request_repaint();
        }
    }
}

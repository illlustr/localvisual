mod components;
mod styles;

use crate::{YtDlpApp, short_codec};
use components::*;
use styles::*;
use eframe::egui;

// Create a UI state struct to manage UI-specific state
pub struct UiState {
    scroll_position: f32,
    last_repaint: std::time::Instant,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            scroll_position: 0.0,
            last_repaint: std::time::Instant::now(),
        }
    }
}

pub fn render_ui(app: &mut YtDlpApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        apply_ui_styling(ui, &app.config.ui);
        
        render_top_bar(app, ui);
        ui.add_space(app.config.ui.padding);
        
        if app.show_settings {
            render_settings_panel(app, ui);
            ui.add_space(app.config.ui.padding);
        }
        
        render_url_input(app, ui);
        ui.add_space(app.config.ui.padding);
        
        if !app.status.is_empty() {
            render_status(app, ui);
        }
        
        if !app.formats.is_empty() {
            render_formats_panel(app, ui);
        }
    });
}

// Move existing render functions to separate files in the components module
// ...existing code...
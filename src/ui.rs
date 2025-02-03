use eframe::egui;
use crate::{YtDlpApp, short_codec};

pub fn render_ui(app: &mut YtDlpApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        show_settings(app, ui);
        show_url_input(app, ui);
        show_action_buttons(app, ui);
        show_status_message(app, ui);
        show_formats_table(app, ui);
    });
}

fn show_settings(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("🎬 LocalVisual")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    show_yt_dlp_path(app, ui);
                    show_download_dir(app, ui);
                });
        });
}

fn show_yt_dlp_path(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    ui.label("YT-DLP Path:");
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut app.yt_dlp_path)
            .on_hover_text("Path to yt-dlp executable");
        if ui.button("📂").on_hover_text("Browse").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                app.yt_dlp_path = path.to_string_lossy().to_string();
            }
        }
    });
    ui.end_row();
}

fn show_download_dir(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    ui.label("Save To:");
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut app.download_dir)
            .on_hover_text("Download directory");
        if ui.button("📂").on_hover_text("Browse").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.download_dir = path.to_string_lossy().to_string();
            }
        }
    });
    ui.end_row();
}

fn show_url_input(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    ui.text_edit_singleline(&mut app.url)
        .on_hover_text("Paste video URL here");
    ui.separator();
}

fn show_action_buttons(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let fetch_btn = ui.add(egui::Button::new("🔍 Fetch Formats"));
        let download_btn = ui.add_enabled(app.selected_format.is_some(), egui::Button::new("⏬ Download"));

        if fetch_btn.clicked() {
            app.fetch_formats();
        }

        if download_btn.clicked() {
            app.download_selected_format();
        }
    });
}

fn show_status_message(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let status_color = if app.status.contains("Error") {
            egui::Color32::RED
        } else if app.status.contains("success") || app.status.contains("completed") {
            egui::Color32::GREEN
        } else {
            egui::Color32::WHITE
        };
        ui.label(egui::RichText::new(&app.status).color(status_color));
    });
}

fn show_formats_table(app: &mut YtDlpApp, ui: &mut egui::Ui) {
    if !app.formats.is_empty() {
        ui.separator();
        ui.heading("Available Formats");

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("formats_grid")
                    .num_columns(6)
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        ui.heading("Select");
                        ui.heading("ID");
                        ui.heading("Type");
                        ui.heading("Resolution");
                        ui.heading("Video");
                        ui.heading("Audio");
                        ui.end_row();

                        for (index, format) in app.formats.iter().enumerate() {
                            ui.radio_value(&mut app.selected_format, Some(index), "");
                            ui.monospace(&format.format_id);
                            ui.label(&format.ext);
                            ui.label(format.resolution.as_deref().unwrap_or("N/A"));
                            ui.label(short_codec(&format.vcodec));
                            // Use as_deref() on the Option to get an &str, or "N/A" if missing.
                            let audio_codec = format.acodec.as_deref().unwrap_or("N/A");
                            ui.label(audio_codec);
                            ui.end_row();
                        }
                    });
            });
    }
}

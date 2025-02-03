use eframe::egui;
use crate::{YtDlpApp, short_codec};

// UI Constants - more compact sizes
const ICON_SIZE: f32 = 24.0;
const ROW_HEIGHT: f32 = 24.0;
const SPACING: f32 = 4.0;

pub fn render_ui(app: &mut YtDlpApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Top bar with title
        ui.horizontal(|ui| {
            ui.heading("LocalVisual");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("⚙").on_hover_text("Settings").clicked() {
                    app.show_settings = !app.show_settings;
                }
            });
        });

        // Settings Panel
        if app.show_settings {
            ui.add_space(SPACING);
            ui.group(|ui| {
                ui.columns(2, |cols| {
                    cols[0].label("YT-DLP Path:");
                    cols[1].horizontal(|ui| {
                        ui.text_edit_singleline(&mut app.yt_dlp_path);
                        if ui.small_button("📂").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                app.yt_dlp_path = path.to_string_lossy().to_string();
                            }
                        }
                    });
                    cols[0].label("Save To:");
                    cols[1].horizontal(|ui| {
                        ui.text_edit_singleline(&mut app.download_dir);
                        if ui.small_button("📂").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                app.download_dir = path.to_string_lossy().to_string();
                            }
                        }
                    });
                });
            });
        }

        ui.add_space(SPACING);

        // URL Input row
        ui.horizontal(|ui| {
            // Paste button
            if ui.small_button("📋").on_hover_text("Paste").clicked() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        app.url = text;
                    }
                }
            }
            
            // URL input
            let available_width = ui.available_width() - (ICON_SIZE + SPACING) * 2.0;
            ui.add_sized(
                egui::Vec2::new(available_width, ROW_HEIGHT),
                egui::TextEdit::singleline(&mut app.url).hint_text("Paste video URL here")
            );
            
            // Clear button
            if ui.small_button("❌").on_hover_text("Clear").clicked() {
                app.url.clear();
                app.formats.clear();
                app.selected_format = None;
                app.status.clear();
            }
        });

        // Action buttons
        if !app.url.is_empty() {
            ui.horizontal(|ui| {
                let button_width = if app.selected_format.is_some() {
                    ui.available_width() / 2.0 - SPACING
                } else {
                    ui.available_width()
                };

                if ui.add_sized(
                    egui::Vec2::new(button_width, ROW_HEIGHT),
                    egui::Button::new("🔍 Fetch Formats")
                ).clicked() {
                    app.fetch_formats();
                }

                if app.selected_format.is_some() {
                    ui.add_space(SPACING * 2.0);
                    if ui.add_sized(
                        egui::Vec2::new(button_width, ROW_HEIGHT),
                        egui::Button::new("⏬ Download")
                    ).clicked() {
                        app.download_selected_format();
                    }
                }
            });
        }

        ui.add_space(SPACING);
        ui.separator();

        // Status Message
        if !app.status.is_empty() {
            let status_color = get_status_color(&app.status);
            ui.colored_label(status_color, &app.status);
            ui.add_space(SPACING);
        }

        // Formats Table
        if !app.formats.is_empty() {
            ui.heading("Available Formats");
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("formats_grid")
                    .num_columns(6)
                    .striped(true)
                    .spacing([SPACING * 2.0, SPACING])
                    .min_col_width(60.0)
                    .show(ui, |ui| {
                        // Table Header
                        ui.strong("Select");
                        ui.strong("ID");
                        ui.strong("Type");
                        ui.strong("Resolution");
                        ui.strong("Video");
                        ui.strong("Audio");
                        ui.end_row();

                        // Table Rows
                        for (index, format) in app.formats.iter().enumerate() {
                            ui.radio_value(&mut app.selected_format, Some(index), "");
                            ui.monospace(&format.format_id);
                            ui.label(&format.ext);
                            ui.label(format.resolution.as_deref().unwrap_or("N/A"));
                            ui.label(short_codec(&format.vcodec));
                            ui.label(format.acodec.as_deref().unwrap_or("N/A"));
                            ui.end_row();
                        }
                    });
            });
        }
    });
}

fn get_status_color(status: &str) -> egui::Color32 {
    if status.contains("Error") {
        egui::Color32::RED
    } else if status.contains("success") || status.contains("completed") {
        egui::Color32::GREEN
    } else {
        egui::Color32::WHITE
    }
}

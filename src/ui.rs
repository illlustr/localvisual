use eframe::egui;
use crate::{YtDlpApp, short_codec};

// UI Constants
const ROW_HEIGHT: f32 = 28.0;
const SPACING: f32 = 2.0;
const MARGIN: f32 = 8.0;
const PADDING: f32 = 8.0;
const ICON_BUTTON_SIZE: f32 = 28.0;

fn text_edit_style(ui: &mut egui::Ui, text: &mut String, hint: Option<&str>, width: f32) -> egui::Response {
    ui.add_sized(
        egui::Vec2::new(width, ROW_HEIGHT),
        egui::TextEdit::singleline(text)
            .margin(egui::vec2(MARGIN, 0.0))
            .hint_text(hint.unwrap_or(""))
            .vertical_align(egui::Align::Center)
    ).on_hover_text(hint.unwrap_or(""))
}

pub fn render_ui(app: &mut YtDlpApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(SPACING, SPACING);
        ui.spacing_mut().window_margin = egui::style::Margin::same(MARGIN);
        ui.set_min_height(ui.available_height());

        // Top bar
        ui.horizontal(|ui| {
            ui.heading("🌐");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let settings_button = egui::Button::new("⚙")
                    .fill(if app.show_settings {
                        ui.style().visuals.selection.bg_fill
                    } else {
                        ui.style().visuals.widgets.inactive.bg_fill
                    });
                
                if ui.add_sized(
                    egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                    settings_button
                ).clicked() {
                    app.show_settings = !app.show_settings;
                }
            });
        });

        ui.add_space(PADDING);

        // Settings Panel
        if app.show_settings {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.set_width(100.0);
                        ui.vertical_centered_justified(|ui| {
                            ui.label("YT-DLP Path:");
                            ui.add_space(ROW_HEIGHT);
                            ui.label("Save To:");
                        });
                    });

                    cols[1].vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = PADDING;
                        ui.horizontal(|ui| {
                            let available = ui.available_width() - (ICON_BUTTON_SIZE + SPACING);
                            text_edit_style(ui, &mut app.yt_dlp_path, Some("Path to yt-dlp executable"), available);
                            if ui.add_sized(
                                egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                                egui::Button::new("📂")
                            ).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    app.yt_dlp_path = path.to_string_lossy().to_string();
                                }
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            let available = ui.available_width() - (ICON_BUTTON_SIZE + SPACING);
                            text_edit_style(ui, &mut app.download_dir, Some("Download destination folder"), available);
                            if ui.add_sized(
                                egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                                egui::Button::new("📂")
                            ).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    app.download_dir = path.to_string_lossy().to_string();
                                }
                            }
                        });
                    });
                });
            });
            ui.add_space(PADDING);
        }

        // URL Input
        ui.horizontal(|ui| {
            if ui.add_sized(
                egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                egui::Button::new("📋")
            ).on_hover_text("Paste").clicked() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        app.url = text;
                    }
                }
            }
            
            ui.add_space(SPACING);
            
            let available = if app.url.is_empty() {
                ui.available_width()
            } else {
                ui.available_width() - (ICON_BUTTON_SIZE * 2.0 + SPACING * 2.0)
            };
            
            text_edit_style(ui, &mut app.url, Some("Paste video URL here"), available);
            
            if !app.url.is_empty() {
                if ui.add_sized(
                    egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                    egui::Button::new("🔍")
                ).on_hover_text("Fetch Formats").clicked() {
                    app.fetch_formats();
                }
                if ui.add_sized(
                    egui::Vec2::new(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
                    egui::Button::new("❌")
                ).on_hover_text("Clear").clicked() {
                    app.url.clear();
                    app.formats.clear();
                    app.selected_format = None;
                    app.status.clear();
                }
            }
        });

        ui.add_space(PADDING);


        // Status Message
        if !app.status.is_empty() {
            let status_color = get_status_color(&app.status);
            ui.vertical_centered(|ui| {
                ui.colored_label(status_color, &app.status);
            });
            ui.add_space(SPACING);
        }

        // Formats Section
        if !app.formats.is_empty() {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    // Sticky header
                    egui::Grid::new("formats_header")
                        .num_columns(6)
                        .spacing([SPACING * 2.0, SPACING])
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| ui.strong("Select"));
                            ui.horizontal_centered(|ui| ui.strong("ID"));
                            ui.horizontal_centered(|ui| ui.strong("Type"));
                            ui.horizontal_centered(|ui| ui.strong("Resolution"));
                            ui.horizontal_centered(|ui| ui.strong("Video"));
                            ui.horizontal_centered(|ui| ui.strong("Audio"));
                            ui.end_row();
                        });

                    // Scrollable area with dynamic height
                    let scroll_height = ui.available_height() - if app.selected_format.is_some() { ROW_HEIGHT + PADDING } else { 0.0 };
                    egui::ScrollArea::vertical()
                        .max_height(scroll_height)
                        .show(ui, |ui| {
                            egui::Grid::new("formats_grid")
                                .num_columns(6)
                                .striped(true)
                                .spacing([SPACING * 2.0, SPACING])
                                .min_col_width(60.0)
                                .show(ui, |ui| {
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

                    // Download button (fixed position)
                    if app.selected_format.is_some() {
                        ui.add_space(PADDING);
                        ui.horizontal(|ui| {
                            if ui.add_sized(
                                egui::Vec2::new(ui.available_width(), ROW_HEIGHT),
                                egui::Button::new("⏬ Download")
                            ).clicked() {
                                app.download_selected_format();
                            }
                        });
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
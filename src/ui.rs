use eframe::egui;
use crate::{YtDlpApp, short_codec};

fn text_edit_style(row_height: f32, margin: f32, ui: &mut egui::Ui, text: &mut String, hint: Option<&str>, width: f32) -> egui::Response {
    ui.add_sized(
        egui::Vec2::new(width, row_height),
        egui::TextEdit::singleline(text)
            .margin(egui::vec2(margin, 0.0))
            .hint_text(hint.unwrap_or(""))
            .vertical_align(egui::Align::Center)
    ).on_hover_text(hint.unwrap_or(""))
}

pub fn render_ui(app: &mut YtDlpApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(app.config.spacing, app.config.spacing);
        ui.spacing_mut().window_margin = egui::style::Margin::same(app.config.margin);
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
                    egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                    settings_button
                ).clicked() {
                    app.show_settings = !app.show_settings;
                }
            });
        });

        ui.add_space(app.config.padding);

        // Settings Panel
        if app.show_settings {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.set_width(140.0);
                        ui.vertical_centered_justified(|ui| {
                            ui.label("YT-DLP Path:");
                            ui.add_space(app.config.row_height);
                            ui.label("Save To:");
                            ui.add_space(app.config.row_height);
                            ui.label("Row Height:");
                            ui.label("Spacing:");
                            ui.label("Margin:");
                            ui.label("Padding:");
                            ui.label("Icon Size:");
                        });
                    });

                    cols[1].vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = app.config.padding;
                        
                        // Path settings
                        ui.horizontal(|ui| {
                            let available = ui.available_width() - (app.config.icon_button_size + app.config.spacing);
                            text_edit_style(
                                app.config.row_height,
                                app.config.margin,
                                ui,
                                &mut app.yt_dlp_path,
                                Some("Path to yt-dlp executable"),
                                available
                            );
                            if ui.add_sized(
                                egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                                egui::Button::new("📂")
                            ).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    app.yt_dlp_path = path.to_string_lossy().to_string();
                                    app.save_config();
                                }
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            let available = ui.available_width() - (app.config.icon_button_size + app.config.spacing);
                            text_edit_style(
                                app.config.row_height,
                                app.config.margin,
                                ui,
                                &mut app.download_dir,
                                Some("Download destination folder"),
                                available
                            );
                            if ui.add_sized(
                                egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                                egui::Button::new("📂")
                            ).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    app.download_dir = path.to_string_lossy().to_string();
                                    app.save_config();
                                }
                            }
                        });

                        // UI settings
                        ui.add(egui::Slider::new(&mut app.config.row_height, 20.0..=40.0).text("px"));
                        ui.add(egui::Slider::new(&mut app.config.spacing, 0.0..=10.0).text("px"));
                        ui.add(egui::Slider::new(&mut app.config.margin, 0.0..=20.0).text("px"));
                        ui.add(egui::Slider::new(&mut app.config.padding, 0.0..=20.0).text("px"));
                        ui.add(egui::Slider::new(&mut app.config.icon_button_size, 20.0..=40.0).text("px"));
                        
                        if ui.button("Save UI Settings").clicked() {
                            app.save_config();
                        }
                    });
                });
            });
            ui.add_space(app.config.padding);
        }

        // URL Input
        ui.horizontal(|ui| {
            if ui.add_sized(
                egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                egui::Button::new("📋")
            ).on_hover_text("Paste").clicked() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        app.url = text;
                    }
                }
            }
            
            ui.add_space(app.config.spacing);
            
            let available = if app.url.is_empty() {
                ui.available_width()
            } else {
                ui.available_width() - (app.config.icon_button_size * 2.0 + app.config.spacing * 2.0)
            };
            
            text_edit_style(
                app.config.row_height,
                app.config.margin,
                ui,
                &mut app.url,
                Some("Paste URL here"),
                available
            );
            
            if !app.url.is_empty() {
                if ui.add_sized(
                    egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                    egui::Button::new("🔍")
                ).on_hover_text("Fetch Formats").clicked() {
                    app.fetch_formats();
                }
                if ui.add_sized(
                    egui::Vec2::new(app.config.icon_button_size, app.config.icon_button_size),
                    egui::Button::new("❌")
                ).on_hover_text("Clear").clicked() {
                    app.url.clear();
                    app.formats.clear();
                    app.selected_format = None;
                    app.status.clear();
                }
            }
        });

        ui.add_space(app.config.padding);


        // Status Message
        if !app.status.is_empty() {
            let status_color = get_status_color(&app.status);
            ui.vertical_centered(|ui| {
                ui.colored_label(status_color, &app.status);
            });
            ui.add_space(app.config.spacing);
        }

        // Formats Section
        if !app.formats.is_empty() {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    // Sticky header
                    egui::Grid::new("formats_header")
                        .num_columns(6)
                        .spacing([app.config.spacing * 2.0, app.config.spacing])
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

                    let scroll_height = ui.available_height() - if app.selected_format.is_some() { app.config.row_height + app.config.padding } else { 0.0 };
                    egui::ScrollArea::vertical()
                        .max_height(scroll_height)
                        .show(ui, |ui| {
                            egui::Grid::new("formats_grid")
                                .num_columns(6)
                                .striped(true)
                                .spacing([app.config.spacing * 2.0, app.config.spacing])
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

                    // Download button
                    if app.selected_format.is_some() {
                        ui.add_space(app.config.padding);
                        ui.horizontal(|ui| {
                            if ui.add_sized(
                                egui::Vec2::new(ui.available_width(), app.config.row_height),
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
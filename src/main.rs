use eframe::egui;
use serde::Deserialize;
use std::process::Command;
use tokio::runtime::Runtime;

#[derive(Debug, Deserialize)]
struct FormatInfo {
    format_id: String,
    ext: String,
    resolution: Option<String>,
    vcodec: String,
    acodec: String,
}

struct YtDlpApp {
    yt_dlp_path: String,
    download_dir: String,
    url: String,
    formats: Vec<FormatInfo>,
    selected_format: Option<usize>,
    status: String,
}

impl Default for YtDlpApp {
    fn default() -> Self {
        Self {
            yt_dlp_path: "yt-dlp.exe".to_string(),
            download_dir: std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            url: String::new(),
            formats: Vec::new(),
            selected_format: None,
            status: String::new(),
        }
    }
}

impl eframe::App for YtDlpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_settings(ui);
            self.show_url_input(ui);
            self.show_action_buttons(ui);
            self.show_status_message(ui);
            self.show_formats_table(ui);
        });
    }
}

impl YtDlpApp {
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("🎬 LocalVisual")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        self.show_yt_dlp_path(ui);
                        self.show_download_dir(ui);
                    });
            });
    }

    fn show_yt_dlp_path(&mut self, ui: &mut egui::Ui) {
        ui.label("YT-DLP Path:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.yt_dlp_path)
                .on_hover_text("Path to yt-dlp executable");
            if ui.button("📂").on_hover_text("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.yt_dlp_path = path.to_string_lossy().to_string();
                }
            }
        });
        ui.end_row();
    }

    fn show_download_dir(&mut self, ui: &mut egui::Ui) {
        ui.label("Save To:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.download_dir)
                .on_hover_text("Download directory");
            if ui.button("📂").on_hover_text("Browse").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.download_dir = path.to_string_lossy().to_string();
                }
            }
        });
        ui.end_row();
    }

    fn show_url_input(&mut self, ui: &mut egui::Ui) {
        ui.text_edit_singleline(&mut self.url)
            .on_hover_text("Paste video URL here");
        ui.separator();
    }

    fn show_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let fetch_btn = ui.add(egui::Button::new("🔍 Fetch Formats"));
            let download_btn = ui.add_enabled(
                self.selected_format.is_some(),
                egui::Button::new("⏬ Download")
            );

            if fetch_btn.clicked() {
                self.fetch_formats();
            }

            if download_btn.clicked() {
                self.download_selected_format();
            }
        });
    }

    fn show_status_message(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let status_color = if self.status.contains("Error") {
                egui::Color32::RED
            } else if self.status.contains("success") || self.status.contains("completed") {
                egui::Color32::GREEN
            } else {
                egui::Color32::WHITE
            };
            ui.label(egui::RichText::new(&self.status).color(status_color));
        });
    }

    fn show_formats_table(&mut self, ui: &mut egui::Ui) {
        if !self.formats.is_empty() {
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

                            for (index, format) in self.formats.iter().enumerate() {
                                ui.radio_value(&mut self.selected_format, Some(index), "");
                                ui.monospace(&format.format_id);
                                ui.label(&format.ext);
                                ui.label(format.resolution.as_deref().unwrap_or("N/A"));
                                ui.label(short_codec(&format.vcodec));
                                ui.label(short_codec(&format.acodec));
                                ui.end_row();
                            }
                        });
                });
        }
    }

    fn fetch_formats(&mut self) {
        self.status = "⏳ Fetching formats...".to_string();
        let url = self.url.clone();
        let yt_dlp_path = self.yt_dlp_path.clone();
        
        Runtime::new().unwrap().block_on(async {
            match Command::new(&yt_dlp_path)
                .args(&[
                    "--no-check-certificate",
                    "--flat-playlist",
                    "--print",
                    "%()j",
                    &url,
                ])
                .output()
            {
                Ok(output) => {
                    let mut formats = Vec::new();
                    for line in String::from_utf8_lossy(&output.stdout).lines() {
                        if let Ok(format) = serde_json::from_str::<FormatInfo>(line) {
                            formats.push(format);
                        }
                    }
                    self.formats = formats;
                    self.status = if self.formats.is_empty() {
                        "❌ No formats found".to_string()
                    } else {
                        format!("✅ Found {} formats", self.formats.len())
                    };
                }
                Err(e) => {
                    self.status = format!("❌ Error: {}", e);
                }
            }
        });
    }

    fn download_selected_format(&mut self) {
        if let Some(index) = self.selected_format {
            if let Some(format) = self.formats.get(index) {
                self.status = "⏳ Downloading...".to_string();
                let url = self.url.clone();
                let yt_dlp_path = self.yt_dlp_path.clone();
                let download_dir = self.download_dir.clone();
                let format_id = format.format_id.clone();
                
                Runtime::new().unwrap().block_on(async {
                    match Command::new(&yt_dlp_path)
                        .args(&[
                            "--no-check-certificate",
                            "-f",
                            &format_id,
                            "-o",
                            &format!("{}/%(title)s.%(ext)s", download_dir),
                            &url,
                        ])
                        .status()
                    {
                        Ok(status) => {
                            self.status = if status.success() {
                                "✅ Download completed!".to_string()
                            } else {
                                "❌ Download failed".to_string()
                            };
                        }
                        Err(e) => {
                            self.status = format!("❌ Error: {}", e);
                        }
                    }
                });
            }
        }
    }
}

fn short_codec(codec: &str) -> String {
    match codec {
        "avc1" | "h264" => "H.264".to_string(),
        "vp9" => "VP9".to_string(),
        "av01" => "AV1".to_string(),
        "mp4a" => "AAC".to_string(),
        "opus" => "Opus".to_string(),
        "none" => "—".to_string(),
        _ => codec.split('.').next().unwrap_or(codec).to_string(),
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        decorated: true,
        ..Default::default()
    };
    
    if let Err(e) = eframe::run_native(
        "LocalVisual",
        options,
        Box::new(|_cc| Box::new(YtDlpApp::default())),
    ) {
        eprintln!("Error: {}", e);
    }
}
mod ui;

use eframe::egui;
use serde::Deserialize;
use std::process::Command;
use tokio::runtime::Runtime;

#[derive(Debug, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub vcodec: String,
    pub acodec: Option<String>, // Now optional to allow missing values.
}

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    pub formats: Vec<FormatInfo>,
}

pub struct YtDlpApp {
    pub yt_dlp_path: String,
    pub download_dir: String,
    pub url: String,
    pub formats: Vec<FormatInfo>,
    pub selected_format: Option<usize>,
    pub status: String,
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
        // Delegate UI rendering to the ui module.
        ui::render_ui(self, ctx);
    }
}

impl YtDlpApp {
    pub fn fetch_formats(&mut self) {
        self.status = "⏳ Fetching formats...".to_string();
        let url = self.url.clone();
        let yt_dlp_path = self.yt_dlp_path.clone();

        Runtime::new().unwrap().block_on(async {
            match Command::new(&yt_dlp_path)
                .args(&[
                    "--no-check-certificate",
                    "--skip-download",
                    "--print-json",
                    &url,
                ])
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    match serde_json::from_str::<VideoInfo>(&output_str) {
                        Ok(video_info) => {
                            self.formats = video_info.formats;
                            self.status = if self.formats.is_empty() {
                                "❌ No formats found".to_string()
                            } else {
                                format!("✅ Found {} formats", self.formats.len())
                            };
                        }
                        Err(e) => {
                            self.status = format!("❌ Error parsing JSON: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status = format!("❌ Error: {}", e);
                }
            }
        });
    }

    pub fn download_selected_format(&mut self) {
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

// Utility function used in the UI.
pub fn short_codec(codec: &str) -> String {
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

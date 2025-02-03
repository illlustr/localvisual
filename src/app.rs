use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc::{self, Sender, Receiver};
use tokio::runtime::Runtime;

#[derive(Debug, Deserialize, Serialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub vcodec: String,
    pub acodec: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub show_settings: bool,
    pub config: Config,
    runtime: Runtime,
    tx: Sender<String>,
    rx: Receiver<String>,
    is_fetching: bool,
    is_downloading: bool,
}

impl Default for YtDlpApp {
    fn default() -> Self {
        let runtime = Runtime::new().unwrap();
        let (tx, rx) = mpsc::channel();
        let config = Config::load();

        Self {
            yt_dlp_path: String::new(),
            download_dir: String::new(),
            config,
            url: String::new(),
            formats: Vec::new(),
            selected_format: None,
            status: String::new(),
            show_settings: false,
            runtime,
            tx,
            rx,
            is_fetching: false,
            is_downloading: false,
        }
    }
}

impl YtDlpApp {
    pub fn fetch_formats(&mut self) {
        if self.is_fetching {
            return;
        }

        self.is_fetching = true;
        self.status = "⏳ Fetching formats...".to_string();
        let url = self.url.clone();
        let yt_dlp_path = self.yt_dlp_path.clone();
        let tx = self.tx.clone();

        self.runtime.spawn(async move {
            let result = Command::new(&yt_dlp_path)
                .args(&["--no-check-certificate", "--skip-download", "--print-json", &url])
                .output();

            match result {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    match serde_json::from_str::<VideoInfo>(&output_str) {
                        Ok(video_info) => {
                            let _ = tx.send(format!("FORMATS:{}", serde_json::to_string(&video_info).unwrap()));
                        }
                        Err(e) => {
                            let _ = tx.send(format!("ERROR:Error parsing JSON: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("ERROR:{}", e));
                }
            }
        });
    }

    pub fn download_selected_format(&mut self) {
        if self.is_downloading {
            return;
        }

        if let Some(index) = self.selected_format {
            if let Some(format) = self.formats.get(index) {
                self.is_downloading = true;
                self.status = "⏳ Downloading...".to_string();
                let url = self.url.clone();
                let yt_dlp_path = self.yt_dlp_path.clone();
                let download_dir = self.download_dir.clone();
                let format_id = format.format_id.clone();
                let tx = self.tx.clone();

                self.runtime.spawn(async move {
                    let result = Command::new(&yt_dlp_path)
                        .args(&[
                            "--no-check-certificate",
                            "-f",
                            &format_id,
                            "-o",
                            &format!("{}/%(title)s.%(ext)s", download_dir),
                            &url,
                        ])
                        .status();

                    match result {
                        Ok(status) => {
                            let msg = if status.success() {
                                "SUCCESS:Download completed!"
                            } else {
                                "ERROR:Download failed"
                            };
                            let _ = tx.send(msg.to_string());
                        }
                        Err(e) => {
                            let _ = tx.send(format!("ERROR:{}", e));
                        }
                    }
                });
            }
        }
    }

    fn process_status_message(&mut self, message: &str) {
        if let Some((status_type, content)) = message.split_once(':') {
            match status_type {
                "FORMATS" => {
                    if let Ok(video_info) = serde_json::from_str::<VideoInfo>(content) {
                        self.formats = video_info.formats;
                        self.status = format!("✅ Found {} formats", self.formats.len());
                    }
                    self.is_fetching = false;
                }
                "SUCCESS" => {
                    self.status = format!("✅ {}", content);
                    self.is_downloading = false;
                }
                "ERROR" => {
                    self.status = format!("❌ {}", content);
                    self.is_fetching = false;
                    self.is_downloading = false;
                }
                _ => {}
            }
        }
    }

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.status = format!("❌ Failed to save config: {}", e);
        }
    }

    pub fn handle_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            self.process_status_message(&message);
        }
    }

    pub fn is_busy(&self) -> bool {
        self.is_fetching || self.is_downloading
    }
}

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

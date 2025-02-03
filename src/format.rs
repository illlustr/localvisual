use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VideoInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub vcodec: String,
    pub acodec: Option<String>,
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

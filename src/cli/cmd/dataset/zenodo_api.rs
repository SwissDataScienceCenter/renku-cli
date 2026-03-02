use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ZenodoLinks {
    pub bucket: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DepositionResponse {
    pub id: u32,
    pub title: String,
    pub state: String,
    pub created: DateTime<Local>,
    pub links: ZenodoLinks,
}

#[derive(Deserialize, Debug)]
pub struct FileResponse {
    pub filename: String,
    pub filesize: f64,
    pub checksum: String,
}

#[derive(Deserialize, Debug)]
pub struct FileUploadResponse {
    pub key: String,
    pub mimetype: String,
    pub checksum: String,
    pub size: u64,
}

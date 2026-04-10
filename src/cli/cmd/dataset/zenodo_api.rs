use chrono::{DateTime, Local};
use serde::Deserialize;
use tabled::Tabled;

#[derive(Deserialize, Debug)]
pub struct ZenodoLinks {
    pub bucket: Option<String>,
}

#[derive(Deserialize, Debug, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct DepositionResponse {
    #[tabled(rename = "ID")]
    pub id: u32,
    pub title: String,
    pub state: String,
    #[tabled(display("display_date"), rename = "Created at")]
    pub created: DateTime<Local>,
    #[tabled(skip)]
    pub links: ZenodoLinks,
}

fn display_date(date: &DateTime<Local>) -> String {
    date.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

#[derive(Deserialize, Debug, Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct FileResponse {
    pub filename: String,
    #[tabled(rename = "Size (bytes)")]
    pub filesize: f64,
    pub checksum: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct FileUploadResponse {
    pub key: String,
    pub mimetype: String,
    pub checksum: String,
    pub size: u64,
}

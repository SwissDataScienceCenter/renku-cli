use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::path::{Path, PathBuf};

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("Unable to read config file {}: {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Unable to write config file {}: {}", path.display(), source))]
    WriteFile {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Unable to parse file {}: {}", path.display(), source))]
    ParseFile {
        source: toml::de::Error,
        path: PathBuf,
    },
    #[snafu(display("The config file could not be serialized"))]
    WriteToml {
        source: toml::ser::Error,
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RenkuProjectConfig {
    /// The base url to the renku platform.
    pub renku_url: String,

    /// Information about the project
    pub project: ProjectInfo,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProjectInfo {
    pub id: String,
    pub namespace: String,
    pub slug: String,
}

impl RenkuProjectConfig {
    pub fn read(file: &Path) -> Result<RenkuProjectConfig, ConfigError> {
        let cnt = std::fs::read_to_string(file).map_err(|e| ConfigError::ReadFile {
            source: e,
            path: file.to_path_buf(),
        });
        cnt.and_then(|c| {
            toml::from_str(&c).map_err(|e| ConfigError::ParseFile {
                source: e,
                path: file.to_path_buf(),
            })
        })
    }

    pub fn write(&self, file: &Path) -> Result<(), ConfigError> {
        if !file.exists() {
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir).map_err(|e| ConfigError::WriteFile {
                    source: e,
                    path: file.to_path_buf(),
                })?;
            }
        }
        let cnt = toml::to_string(self).map_err(|e| ConfigError::WriteToml {
            source: e,
            path: file.to_path_buf(),
        });

        cnt.and_then(|c| {
            std::fs::write(file, c).map_err(|e| ConfigError::WriteFile {
                source: e,
                path: file.to_path_buf(),
            })
        })
    }
}

#[test]
fn write_and_read_config() {
    let data = RenkuProjectConfig {
        renku_url: "http://renkulab.io".into(),
        project: ProjectInfo {
            id: "abc123".into(),
            namespace: "my-ns".into(),
            slug: "projecta".into(),
        },
    };
    let tmp = std::env::temp_dir();
    let target = tmp.join("test.conf");
    data.write(&target).unwrap();
    let from_file = RenkuProjectConfig::read(&target).unwrap();
    std::fs::remove_file(&target).unwrap();
    assert_eq!(data, from_file);
}

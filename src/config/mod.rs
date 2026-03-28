use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};

mod log_config;
pub use log_config::LogConfig;

pub static CONFIG: OnceLock<ServerConfig> = OnceLock::new();

pub fn init() {
    let path = Env::var("APP_CONFIG").unwrap_or("config.toml".to_owned());

    // Create default config if missing
    let config = if !Path::new(&path).exists() {
        let config = ServerConfig::default();
        match toml::to_string_pretty(&config) {
            Ok(serialized) => {
                if let Err(e) = fs::write(path, serialized) {
                    tracing::error!("Failed to write default config file: {}", e)
                }
            }
            Err(e) => {
                tracing::error!("Failed to generate default config: {}", e)
            }
        }
        config
    }
    // Load config file
    else {
        let raw_config = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("APP_").global());

        match raw_config.extract::<ServerConfig>() {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "It looks like your config is invalid. The following error occurred: {e}"
                );
                std::process::exit(1);
            }
        }
    };

    crate::config::CONFIG
        .set(config)
        .expect("config should be set");
}
pub fn get() -> &'static ServerConfig {
    CONFIG.get().expect("config should be set")
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    pub log: LogConfig,
    pub tls: TlsConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TlsConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_empty_str")]
    pub cert_path: String,
    #[serde(default = "default_empty_str")]
    pub key_path: String,
}

#[allow(dead_code)]
pub fn default_false() -> bool {
    false
}
#[allow(dead_code)]
pub fn default_true() -> bool {
    true
}
#[allow(dead_code)]
pub fn default_empty_str() -> String {
    "".to_owned()
}

fn default_listen_addr() -> String {
    "127.0.0.1:8008".into()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),

            log: LogConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use tokio::fs;
use tokio::fs::OpenOptions as TokioOpenOptions;
use tokio::io::Error as IoError;
use tokio::io::ErrorKind;

// Path to the Config file, see EbbflowDaemonConfig in the config module.
#[cfg(target_os = "linux")]
lazy_static! {
    pub static ref CONFIG_PATH: String = "/etc/ebbflow".to_string();
}
#[cfg(target_os = "macos")]
pub const CONFIG_PATH: &str = "/usr/local/etc/ebbflow";
#[cfg(windows)]
lazy_static! {
    pub static ref CONFIG_PATH: String = { "\\Program Files\\ebbflow".to_string() };
}

pub fn config_path_root() -> String {
    CONFIG_PATH.to_string()
}

#[cfg(windows)]
pub fn config_file_full() -> String {
    format!("{}\\{}", config_path_root(), CONFIG_FILE)
}

#[cfg(not(windows))]
pub fn config_file_full() -> String {
    format!("{}/{}", config_path_root(), CONFIG_FILE)
}

#[cfg(windows)]
pub fn key_file_full() -> String {
    format!("{}\\{}", config_path_root(), KEY_FILE)
}

#[cfg(not(windows))]
pub fn key_file_full() -> String {
    format!("{}/{}", config_path_root(), KEY_FILE)
}

pub const CONFIG_FILE: &str = "config.yaml";
pub const KEY_FILE: &str = "host.key";

#[derive(Debug)]
pub enum ConfigError {
    Parsing,
    FileNotFound,
    FilePermissions,
    Empty,
    Unknown(String),
}

pub async fn getkey() -> Result<String, ConfigError> {
    let s = fs::read_to_string(key_file_full()).await?;
    if s.is_empty() {
        return Err(ConfigError::Empty);
    }

    Ok(s.trim().to_string())
}

pub async fn setkey(k: &str) -> Result<(), ConfigError> {
    Ok(fs::write(key_file_full(), k.trim().as_bytes()).await?)
}

impl From<IoError> for ConfigError {
    fn from(ioe: IoError) -> Self {
        match ioe.kind() {
            ErrorKind::NotFound => ConfigError::FileNotFound,
            ErrorKind::PermissionDenied => ConfigError::FilePermissions,
            _ => ConfigError::Unknown(format!("Unexepected error reading config file {:?}", ioe)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PossiblyEmptyEbbflowDaemonConfig {
    Empty,
    EbbflowDaemonConfig(EbbflowDaemonConfig),
}

/// Configuration for Ebbflow. Will be parsed to/from a YAML file located at
/// - /etc/ebbflow for Linux
/// - TBD for Windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbbflowDaemonConfig {
    /// A list of endpoints to host, see Endpoint
    pub endpoints: Vec<Endpoint>,
    /// SSH Config overrides, not needed
    pub ssh: Option<Ssh>,
}

impl EbbflowDaemonConfig {
    pub async fn check_permissions() -> Result<(), ConfigError> {
        let mut std = OpenOptions::new();
        std.write(true).create(true);
        let options = TokioOpenOptions::from(std);

        options.open(config_file_full()).await?;
        options.open(key_file_full()).await?;
        Ok(())
    }

    pub async fn load_from_file_or_new() -> Result<EbbflowDaemonConfig, ConfigError> {
        let cfg = match Self::load_from_file().await {
            Ok(existing) => existing,
            Err(e) => match e {
                ConfigError::Empty | ConfigError::FileNotFound => EbbflowDaemonConfig {
                    endpoints: vec![],
                    ssh: None,
                },
                _ => return Err(e),
            },
        };
        Ok(cfg)
    }

    pub async fn load_from_file() -> Result<EbbflowDaemonConfig, ConfigError> {
        let filebytes = fs::read(config_file_full()).await?;

        let parsed: EbbflowDaemonConfig = match serde_yaml::from_slice(&filebytes[..]) {
            Ok(p) => match p {
                PossiblyEmptyEbbflowDaemonConfig::Empty => return Err(ConfigError::Empty),
                PossiblyEmptyEbbflowDaemonConfig::EbbflowDaemonConfig(c) => c,
            },
            Err(_e) => {
                info!("Error parsing configuration file");
                return Err(ConfigError::Parsing);
            }
        };

        Ok(parsed)
    }
    pub async fn save_to_file(&self) -> Result<(), ConfigError> {
        let b: String = match serde_yaml::to_string(self) {
            Ok(s) => s,
            Err(_e) => {
                info!("Error parsing current configuration into a YAML file");
                return Err(ConfigError::Parsing);
            }
        };

        Ok(fs::write(config_file_full(), b.as_bytes()).await?)
    }
}

/// An Endpoint to host. Provide the DNS name, and the local port. Optionally override the local address,
/// which defaults to 127.0.0.1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endpoint {
    /// The port your application runs on
    pub port: u16,
    /// The DNS name of the endpoint being hosted
    pub dns: String,
    /// the maximum amount of open connections, defaults to 200
    pub maxconns: u16,
    /// the maxmimum amount of idle connections to Ebbflow, will be capped at X
    pub maxidle: u16,
    /// Is this endpoint enabled or disabled?
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ssh {
    /// the maximum amount of open connections
    pub maxconns: u16,
    /// The local port, defaults to 22
    pub port: u16,
    /// Is SSH enabled?
    pub enabled: bool,
    /// the maxmimum amount of idle connections to Ebbflow, will be capped at X
    pub maxidle: u16,
    /// The hostname to use as the target, defaults the OS provided Hostname
    pub hostname_override: Option<String>,
}

impl Ssh {
    pub fn new(enabled: bool, hostname: Option<String>) -> Ssh {
        Self {
            maxconns: 20,
            port: 22,
            enabled,
            hostname_override: hostname,
            maxidle: 5,
        }
    }
}

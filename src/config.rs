//! Configs/Settings useds to access the datasource, setup
//! the template and send the result

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub send: ConfigSend,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigSend {
    pub mail: Option<ConfigMail>,
    #[serde(default)]
    pub stdout: bool,
}

/// Smtp confs
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigMail {
    pub from: String,
    pub to: String,
    pub host: String,
    pub subject: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

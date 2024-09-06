//! Configs/Settings useds to access the datasource, setup
//! the template and send the result

use crate::{
    presentation::formats::OutputFormat, send::MailServer, source::{Query, Source}
};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub source: Source,
    pub send: ConfigSend,
    pub title: String,
    pub querys: Vec<Query>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigSend {
    pub mail: Option<MailServer>,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub format: OutputFormat,
}

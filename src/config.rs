//! Configs/Settings useds to access the datasource, setup
//! the template and send the result

use crate::{
    presentation::{
        charts::ChartComponent, formats::OutputFormat, table::TableComponent, Component,
    },
    send::MailServer,
    source::{Query, Source},
    value::Field,
};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub source: Source,
    pub send: ConfigSend,
    pub title: String,
    pub querys: Vec<ConfigQuery>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigQuery {
    pub title: String,
    pub sql: String,
    pub fields: Vec<Field>,
    #[serde(default)]
    pub chart: Option<ChartComponent>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ConfigSend {
    pub mail: Option<MailServer>,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub format: OutputFormat,
}

impl ConfigQuery {
    pub fn to_query(&self) -> Query {
        Query {
            sql: self.sql.clone(),
            title: self.title.clone(),
            fields: self.fields.clone(),
        }
    }
}

pub fn to_querys(querys: Vec<ConfigQuery>) -> Vec<(Query, Option<ChartComponent>)> {
    querys
        .into_iter()
        .map(|q| (q.to_query(), q.chart))
        .collect()
}

pub fn find_component(
    querys: Vec<(Query, Option<ChartComponent>)>,
    q: Query,
) -> Box<dyn Component> {
    let chart = querys
        .iter()
        .find(|(q2, _)| q2 == &q)
        .map(|(_, c)| c.clone())
        .flatten();

    match chart {
        Some(e) => Box::new(e),
        _ => Box::new(TableComponent {}),
    }
}

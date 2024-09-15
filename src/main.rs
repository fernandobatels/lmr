/// lmr - Lightweight email report tool
use clap::{crate_authors, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::*;
use simplelog::*;
use std::fs;

mod config;
mod presentation;
mod send;
mod source;
mod value;

use config::Config;

#[derive(Parser, Debug)]
#[command(help_template = "\
{before-help}lmr {version}
Lightweight email report tool

{author}
https://github.com/fernandobatels/lmr

{usage-heading} {usage}

{all-args}{after-help}")]
#[command(version)]
#[command(author = crate_authors!())]
struct Args {
    /// Yaml config file
    pub config: String,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    TermLogger::init(
        args.verbose.log_level_filter(),
        ConfigBuilder::new().set_time_format_rfc3339().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .map_err(|e| format!("Logger init failed: {}", e.to_string()))?;

    debug!("Loading the config file: {}", args.config);

    let sconfig = fs::read_to_string(args.config)
        .map_err(|e| format!("Config file not loaded: {}", e.to_string()))?;

    debug!("Parsing the config file");

    let config = serde_yaml::from_str::<Config>(&sconfig)
        .map_err(|e| format!("Config file not parsed: {}", e.to_string()))?;

    let querys = config::to_querys(config.querys);
    let lquerys = querys.iter().map(|q| q.0.clone()).collect::<Vec<_>>();

    let data = source::fetch(config.source, lquerys).await?;

    let mut ndata = vec![];
    for (q, r) in data {
        let chart = config::find_component(querys.clone(), q.clone());
        ndata.push((q, chart, r));
    }

    let content = presentation::present_as(ndata, config.title.clone(), config.send.format)?;

    if config.send.stdout {
        send::to_stdout(&content).await?;
    }

    if let Some(set) = config.send.mail {
        send::to_mail(set, config.title, &content).await?;
    }

    Ok(())
}

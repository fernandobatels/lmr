/// lmr - Lightweight email report tool
use clap::Parser;
use log::*;
use simplelog::*;
use std::fs;
use clap_verbosity_flag::{Verbosity, InfoLevel};

mod config;
mod generate;
mod send;

use config::Config;

/// lmr - Lightweight email report tool
#[derive(Parser, Debug)]
#[command(version, about, author)]
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
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .build(),
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

    let data = generate::DataExported {
        is_html: false,
        content: "?????".to_string(),
    };

    if config.send.stdout {
        send::to_stdout(&data).await?;
    }

    if let Some(set) = config.send.mail {
        send::to_mail(set, &data).await?;
    }

    Ok(())
}

use argopt;
use std::fs;

mod config;
mod send;
mod generate;

use config::Config;

#[argopt::cmd]
#[tokio::main]
async fn main(
    /// Yaml config file
    config: String
) -> Result<(), String> {

    let sconfig = fs::read_to_string(config)
        .map_err(|e| format!("Config file not loaded: {}", e.to_string()))?;

    let config = serde_yaml::from_str::<Config>(&sconfig)
        .map_err(|e| format!("Config file not parsed: {}", e.to_string()))?;

    let data = generate::DataExported {
        is_html: false,
        content: "?????".to_string()
    };

    if config.send.stdout {
        send::to_stdout(&data).await?;
    }

    if let Some(set) = config.send.mail {
        send::to_mail(set, &data).await?;
    }

    Ok(())
}

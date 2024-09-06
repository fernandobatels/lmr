//! Send/Output api

use log::*;
use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use serde::Deserialize;

use crate::presentation::DataPresented;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct MailServer {
    pub from: String,
    pub to: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

/// Send the exported data to STDOUT
pub async fn to_stdout(dt: &DataPresented) -> Result<(), String> {
    println!("{}", dt.content);

    Ok(())
}

/// Send the exported data to email
pub async fn to_mail(config: MailServer, title: String, dt: &DataPresented) -> Result<(), String> {
    info!("Sending as email to {}", config.to);

    let mb = MessageBuilder::new()
        .from(("smrtool".to_string(), config.from))
        .to(config.to)
        .subject(title);

    let message = if dt.is_html {
        mb.html_body(dt.content.clone())
    } else {
        mb.text_body(dt.content.clone())
    };

    let mut conn = SmtpClientBuilder::new(config.host, config.port)
        .implicit_tls(false)
        .credentials((config.user, config.pass))
        .connect()
        .await
        .map_err(|e| format!("SMTP connect failed: {}", e.to_string()))?;

    conn.send(message)
        .await
        .map_err(|e| format!("SMTP send failed: {}", e.to_string()))?;

    Ok(())
}

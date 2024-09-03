pub mod smtp;
pub mod azure;
pub mod mail;

use clap::{command, Parser};
use log::{error, info};
use azure::AzureMailClient;
use anyhow::Result;
use smtp::SmtpServer;
use std::net::TcpListener;

/// Listens for SMTP clients, reads e-mails and then delivers them using Azure Communication Services
#[derive(Debug, Parser)]
#[command(version, author = "Andres Olivares")]
struct Args {
    /// Address to bind to (defaults to 127.0.0.1:25)
    #[arg(long, short = 'a', value_name = "IP:PORT")]
    address: Option<String>,

    /// Domain name (defaults to smtp.domain.com)
    #[arg(long, short = 'd', value_name = "DOMAIN")]
    domain: Option<String>,

    /// ACS Endpoint (eg. https://<my_acs_account>.unitedstates.communication.azure.com)
    #[arg(long, short = 'e', value_name = "ACS_ENDPOINT")]
    endpoint: String,

    /// ACS Access Key
    #[arg(long, short = 'k', value_name = "ACS_ACCESS_KEY")]
    access_key: String,

    /// Log level (eg. trace, debug, info, warn, error)
    #[arg(long, short = 'l', value_name = "LOG_LEVEL")]
    log_level: Option<String>
}

fn main() -> Result<()>
{
    let args = Args::parse();

    let addr = match args.address {
        Some(a) => a,
        None => "127.0.0.1:25".to_string()
    };

    let domain = match args.domain {
        Some(d) => d,
        None => "smtp.domain.com".to_string()
    };

    let log_level = match args.log_level {
        Some(ll) => ll,
        None => "info".to_string()
    };

    let mut builder = pretty_env_logger::formatted_builder();
    
    builder.parse_filters(&log_level);
    builder.init();

    let listener = TcpListener::bind(addr)?;

    info!("Simple SMTP Relay for {domain} listening on {}", listener.local_addr().unwrap());

    loop
    {
        let (stream, addr) = listener.accept()?;        
        
        let domain = domain.clone();
        let endpoint = args.endpoint.clone();
        let access_key = args.access_key.clone();

        info!("Accepted connection from {addr}");

        std::thread::spawn(move || {
            let mut smtp = SmtpServer::new(&domain, &stream, Box::new(move |mail| {
                let from = match &mail.from {
                    Some(f) => f,
                    None => {
                        error!("Received mail with no FROM address");
                        return;
                    }
                };

                let data = match &mail.data {
                    Some(d) => d,
                    None => {
                        error!("Received mail with no data");
                        return;
                    }
                };

                info!("Received mail FROM: {} TO: {}", from, mail.to.join(","));

                let msg = mail::from_mime(from.clone(), &mail.to, &data);
                let client = AzureMailClient::new(&endpoint, &access_key);

                if let Err(e) = client.send_mail(&msg) {
                    error!("Error sending mail: {}", e);
                }
            }));

            if let Err(e) = smtp.start() {
                error!("Error starting SMTP server: {}", e);
            }
        });
    }
}

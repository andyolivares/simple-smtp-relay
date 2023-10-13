pub mod smtp;
pub mod azure;
pub mod mail;

use log::info;
use azure::AzureMailClient;
use mail::AzureMailConverter;
use anyhow::Result;
use smtp::SmtpServer;
use std::{env::args, net::TcpListener};

const ENDPOINT: &str = "ACS_ENDPOINT";
const ACCESS_KEY: &str = "ACS_ACCESS_KEY";

fn main() -> Result<()>
{
    pretty_env_logger::init();

    let addr = args()
        .nth(1)
        .unwrap_or("127.0.0.1:25".to_string());

    let domain = args()
        .nth(2)
        .unwrap_or("smtp.domain.com".to_string());

    let endpoint = std::env::var(ENDPOINT).unwrap_or("".to_string());
    let access_key = std::env::var(ACCESS_KEY).unwrap_or("".to_string());
    let listener = TcpListener::bind(addr)?;

    info!("Simple SMTP Relay for {domain} listening at {}", listener.local_addr().unwrap());

    loop
    {
        let (stream, addr) = listener.accept()?;
        let domain = domain.clone();
        let endpoint = endpoint.clone();
        let access_key = access_key.clone();

        info!("Accepted connection from {addr}");

        std::thread::spawn(move || {
            let mut smtp = SmtpServer::new(domain, stream, Box::new(move |mail| {
                info!("Received mail FROM: {} TO: {}", mail.from, mail.to.join(","));

                let msg = AzureMailConverter::from_mime(mail.from, mail.to, &mail.data);
                let client = AzureMailClient::new(&endpoint, &access_key);

                client.send_mail(&msg);
            }));

            smtp.start()
                .unwrap_or_default();
        });
    }
}

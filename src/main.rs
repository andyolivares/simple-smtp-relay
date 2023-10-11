pub mod smtp;

use log::info;
use anyhow::Result;
use smtp::SmtpServer;
use std::io::Write;
use std::{env::args, net::TcpListener};
use std::fs::File;
use uuid::Uuid;

fn main() -> Result<()>
{
    pretty_env_logger::init();

    let addr = args()
        .nth(1)
        .unwrap_or("127.0.0.1:25".to_string());

    let domain = args()
        .nth(2)
        .unwrap_or("smtp.domain.com".to_string());

    let listener = TcpListener::bind(addr)?;

    info!("Simple SMTP Relay for {domain} listening at {}", listener.local_addr().unwrap());

    loop
    {
        let (stream, addr) = listener.accept()?;
        let domain = domain.clone();

        info!("Accepted connection from {addr}");

        std::thread::spawn(move || {
            let mut smtp = SmtpServer::new(domain, stream, Box::new(move |from, to, data| {
                info!("Received mail FROM: {} TO: {}", from, to.join(","));

                let name = format!("{}.mail", Uuid::new_v4().to_string());
                let mut file = File::create(name).unwrap();

                write!(file, "FROM: {}\r\n", from).unwrap();
                write!(file, "TO: {}\r\n", to.join(",")).unwrap();
                write!(file, "\r\n{}", data.join("")).unwrap();
            }));

            smtp.start()
                .unwrap_or_default();
        });
    }
}

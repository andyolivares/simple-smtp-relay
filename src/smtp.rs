/* Simple SMTP Server Implementation */

use log::{trace, debug, warn, error};
use std::net::TcpStream;
use std::io::{BufReader, BufRead, BufWriter, Write};
use anyhow::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
enum SmtpState {
    Fresh,
    Greeted,
    Rcpt,
    Data
}

pub type NewMailCallback = dyn Fn(String, Vec<String>, Vec<String>);

pub struct SmtpServer {
    state: SmtpState,
    ehlo_greeting: String,
    stream: TcpStream,
    callback: Box<NewMailCallback>,
    to: Vec<String>,
    from: String,
    data: Vec<String>
}

impl SmtpServer {

    const HELLO: &[u8] = b"220 Simple SMTP Relay Server\r\n";
    const OK: &[u8] = b"250 OK\r\n";
    const AUTH_OK: &[u8] = b"235 OK\r\n";
    const SEND_DATA: &[u8] = b"354 End data with <CR><LF>.<CR><LF>\r\n";
    const BYE: &[u8] = b"221 Bye\r\n";
    const EMPTY: &[u8] = &[];
    const EMPTY_STR: &str = "";

    pub fn new(domain: String, stream: TcpStream, callback: Box<NewMailCallback>) -> Self {
        let ehlo_greeting = format!("250-{domain} Hello {domain}\r\n250 AUTH PLAIN LOGIN\r\n");

        Self {
            state: SmtpState::Fresh,
            ehlo_greeting,
            stream,
            callback,
            to: Vec::new(),
            from: String::new(),
            data: Vec::new()
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let mut reader = BufReader::new(self.stream.try_clone().unwrap());
        let mut writer = BufWriter::new(self.stream.try_clone().unwrap());

        writer.write_all(SmtpServer::HELLO)?;
        writer.flush()?;

        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;

            if n == 0 {
                debug!("Client disconnected (or EOF)");
                break;
            }

            trace!("Read line: {line}");

            let resp = self.handle_line(line.as_str())?;

            match resp {
                SmtpServer::BYE => {
                    trace!("Received BYE response, shutting down");

                    writer.write_all(resp)?;
                    writer.flush()?;

                    self.stream.shutdown(std::net::Shutdown::Both)?;
                    
                    break;
                },
                SmtpServer::EMPTY => {
                    trace!("Response is empty, does nothing");
                },
                _ => {
                    writer.write_all(resp)?;
                    writer.flush()?;
                }
            };
        }

        Ok(())
    }

    fn handle_line(&mut self, line: &str) -> Result<&[u8]> {
        let arr: Vec<&str> = line
            .split(|c| char::is_whitespace(c) || c == ':')
            .filter(|s| !s.is_empty())
            .collect();

        let command = arr.get(0).unwrap_or(&SmtpServer::EMPTY_STR).to_string();
        let state = self.state.clone();

        match (command.to_lowercase().as_str(), state) {
            ("helo", SmtpState::Fresh) => {
                debug!("Got HELO");
                self.state = SmtpState::Greeted;
                Ok(SmtpServer::OK)
            },
            ("ehlo", SmtpState::Fresh) => {
                debug!("Got EHLO");
                self.state = SmtpState::Greeted;
                Ok(self.ehlo_greeting.as_bytes())
            },
            ("noop", _) | ("help", _) | ("info", _) | ("vrfy", _) | ("expn", _) => {
                debug!("Got command: {command}");
                Ok(SmtpServer::OK)
            },
            ("rset", _) => {
                debug!("Resetting");
                self.state = SmtpState::Fresh;
                Ok(SmtpServer::OK)
            },
            ("auth", _) => {
                debug!("Acknowledging AUTH");
                Ok(SmtpServer::AUTH_OK)
            },
            ("mail", SmtpState::Greeted) => {
                let mail = arr.get(2)
                    .unwrap_or(&SmtpServer::EMPTY_STR)
                    .to_string()
                    .to_lowercase();

                if mail.is_empty() {
                    error!("Received empty FROM address");
                    Ok(SmtpServer::EMPTY)
                } else {
                    debug!("Mail from: {}", mail);

                    self.from = mail.to_string();
                    self.state = SmtpState::Rcpt;

                    Ok(SmtpServer::OK)
                }
            },
            ("rcpt", SmtpState::Rcpt) => {
                let mail = arr.get(2)
                    .unwrap_or(&SmtpServer::EMPTY_STR)
                    .to_string()
                    .to_lowercase();

                if mail.is_empty() {
                    error!("Received empty TO address");
                    Ok(SmtpServer::EMPTY)
                } else {
                    debug!("Mail to: {}", mail);

                    self.to.push(mail);

                    Ok(SmtpServer::OK)
                }
            },
            ("data", SmtpState::Rcpt) => {
                debug!("Awaiting for data");
                self.state = SmtpState::Data;

                Ok(SmtpServer::SEND_DATA)
            },
            (".", SmtpState::Data) => {
                debug!("Data end");
                self.fire_callback();

                Ok(SmtpServer::OK)
            },
            ("quit", SmtpState::Data) => {
                debug!("Got QUIT");
                self.fire_callback();

                Ok(SmtpServer::BYE)
            },
            (_, SmtpState::Data) => {
                debug!("Received data line");
                self.data.push(line.to_string());

                Ok(SmtpServer::EMPTY)
            },            
            ("quit", _) => {
                debug!("Got QUIT");

                Ok(SmtpServer::BYE)
            },
            _ => {
                warn!("Unexpected command: {command}");
                
                Ok(SmtpServer::EMPTY)
            }
        }
    }

    fn clear(&mut self) {
        self.state = SmtpState::Fresh;
        
        self.to = Vec::new();
        self.from = String::new();
        self.data = Vec::new();
    }

    fn fire_callback(&mut self) {
        //info!("Received mail FROM: {} TO: {}", self.from, self.to.join(","));

        let cb = self.callback.as_mut();

        cb(self.from.clone(), self.to.clone(), self.data.clone());

        self.clear();
    }

}
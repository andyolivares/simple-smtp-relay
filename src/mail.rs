use crate::azure::{AzureMailMessage, AzureMailRecipients, AzureMailAddress, AzureMailContent};
use std::collections::HashMap;
use mailparse::parse_mail;

pub struct AzureMailConverter {}

impl AzureMailConverter {

    pub fn from_mime(sender_address: String, to: Vec<String>, data: &String) -> AzureMailMessage {

        let mail = parse_mail(data.as_bytes()).unwrap();

        let headers: HashMap<String, String> = mail.headers
            .iter()
            .map(|h| (h.get_key(), h.get_value()))
            .collect();

        let recipients = AzureMailRecipients {
            to: to.iter().map(|r| AzureMailAddress::new(r.to_string())).collect(),
            cc: Option::None,
            bcc: Option::None
        };

        let subject = headers.get("Subject").unwrap_or(&String::from("")).to_string();
        let body = mail.get_body().unwrap();
        let content = match mail.ctype.mimetype.as_str() {
            "text/html" => AzureMailContent { subject, html: Option::Some(body), plain_text: Option::None },
            "text/plain" => AzureMailContent { subject, html: Option::None, plain_text: Option::Some(body) },
            _ => AzureMailContent { subject, plain_text: Option::Some(format!("Unsupported Content Type\r\n\r\n{}", body)), html: Option::None }
        };

        AzureMailMessage {
            sender_address,
            reply_to: Option::None,
            headers: Option::Some(headers),
            recipients,
            content
        }

    }

}
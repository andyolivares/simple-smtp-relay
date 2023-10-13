use crate::azure::{AzureMailMessage, AzureMailRecipients, AzureMailAddress, AzureMailContent};
use std::collections::HashMap;
use mailparse::{parse_mail, ParsedMail};
use log::trace;

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
        let html = AzureMailConverter::get_content(&mail, "text/html".to_string());
        let plain_text = AzureMailConverter::get_content(&mail, "text/plain".to_string());

        trace!("HTML: {}", html);
        trace!("Plain Text: {}", plain_text);

        let html = match html.len() {
            0 => Option::None,
            _ => Option::Some(html)
        };

        let plain_text = match plain_text.len() {
            0 => Option::None,
            _ => Option::Some(plain_text)
        };

        let msg = AzureMailMessage {
            sender_address,
            reply_to: Option::None,
            headers: Option::Some(headers),
            recipients,
            content: AzureMailContent {
                subject,
                plain_text,
                html
            }
        };

        trace!("Mail message: {:#?}", msg);

        msg

    }

    fn get_content(mail: &ParsedMail, mimetype: String) -> String {
        let body = mail.get_body().unwrap_or("".to_string());

        if mail.ctype.mimetype == mimetype && body.len() != 0 {
            return body;
        } else {
            for pm in mail.parts() {
                let body = AzureMailConverter::get_content(pm, mimetype.clone());

                if body.len() != 0 {
                    return body;
                }
            }
        }

        "".to_string()
    }

}
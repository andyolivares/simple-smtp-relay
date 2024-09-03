use crate::azure::{AzureMailMessage, AzureMailRecipients, AzureMailAddress, AzureMailContent};
use std::collections::HashMap;
use mailparse::{parse_mail, ParsedMail};
use log::trace;

pub fn from_mime(sender_address: String, to: &Vec<String>, data: &String) -> AzureMailMessage {
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

    let subject = match headers.get("Subject") {
        Some(s) => s.clone(),
        None => "".to_string()
    };

    let html = get_content(&mail, &"text/html".to_string());
    let plain_text = get_content(&mail, &"text/plain".to_string());

    trace!("HTML: {}", html);
    trace!("Plain Text: {}", plain_text);

    let html = match html.is_empty() {
        true => Option::None,
        false => Option::Some(html)
    };

    let plain_text = match plain_text.is_empty() {
        true => Option::None,
        false => Option::Some(plain_text)
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

fn get_content(mail: &ParsedMail, mimetype: &String) -> String {
    let body = mail.get_body().unwrap_or("".to_string());

    if mail.ctype.mimetype.eq(mimetype) && body.len() != 0 {
        return body;
    } else {
        if mail.subparts.len() > 0 {
            for pm in mail.subparts.iter() {
                let body = get_content(pm, mimetype);

                if body.len() != 0 {
                    return body;
                }
            }
        }
    }

    "".to_string()
}
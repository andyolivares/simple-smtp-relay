use std::collections::HashMap;
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use url::{Position, Url};
use serde::Serialize;
use base64::{Engine, alphabet};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use hmac_sha256::{Hash, HMAC};
use chrono::Utc;
use reqwest::blocking::Client;

#[derive(Serialize, Debug)]
pub struct AzureMailAddress {
    pub address: String,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>
}

#[derive(Serialize, Debug)]
pub struct AzureMailRecipients {
    pub to: Vec<AzureMailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<AzureMailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<AzureMailAddress>>
}

#[derive(Serialize, Debug)]
pub struct AzureMailContent {
    pub subject: String,
    #[serde(rename = "plainText", skip_serializing_if = "Option::is_none")]
    pub plain_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>
}

#[derive(Serialize, Debug)]
pub struct AzureMailMessage {
    #[serde(rename = "senderAddress")]
    pub sender_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    pub recipients: AzureMailRecipients,
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<AzureMailAddress>,
    pub content: AzureMailContent
}

impl AzureMailAddress {
    pub fn new(address: String) -> Self {
        Self {
            address,
            display_name: Option::None
        }
    }

    pub fn with_display_name(address: String, display_name: String) -> Self {
        Self {
            address,
            display_name: Option::Some(display_name)
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AzureMailClient {
    endpoint: String,
    access_key: String
}

impl AzureMailClient {

    const RFC1123: &'static str = "%a, %d %b %Y %H:%M:%S GMT";
    
    pub fn new(endpoint: &str, access_key: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            access_key: access_key.to_string()
        }
    }

    pub fn send_mail(&self, body: &AzureMailMessage) -> Result<()> {
        let mut url = match Url::parse(self.endpoint.as_str()) {
            Ok(u) => u,
            Err(e) => return Err(Error::msg(e.to_string()))
        };

        url.set_query(Option::Some("api-version=2023-03-31"));

        let path_and_query = &url[Position::BeforePath..];
        
        let host = match url.host_str() {
            Some(h) => h,
            None => return Err(Error::msg("Unable to parse host"))
        };

        let body = match serde_json::to_string(body) {
            Ok(b) => b,
            Err(e) => return Err(Error::msg(e.to_string()))
        };

        trace!("URL: {}", url);
        trace!("Content: {}", body);

        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let mut hash = Hash::new();
        
        hash.update(body.as_bytes());

        let content_hash = e.encode(hash.finalize());
        let date = Utc::now().format(AzureMailClient::RFC1123).to_string();
        let str_to_sign = format!("POST\n{}\n{};{};{}", path_and_query, date, host, content_hash);
        let signature = match self.compute_signature(&str_to_sign) {
            Ok(s) => s,
            Err(e) => return Err(Error::msg(e.to_string()))
        };
        let auth = format!("HMAC-SHA256 SignedHeaders=x-ms-date;host;x-ms-content-sha256&Signature={}", signature);
        let guid = uuid::Uuid::new_v4().to_string();

        let client = Client::new();
        let res = client
            .post(url.to_string())
            .header("Authorization", auth)
            .header("x-ms-date", &date)
            .header("x-ms-content-sha256", content_hash)
            .header("Repeatability-Request-Id", guid)
            .header("Repeatability-First-Sent", &date)
            .header("Content-Type", "application/json")
            .body(body)
            .send();

        if let Err(e) = res {
            return Err(Error::msg(e.to_string()));
        }

        let res = res.unwrap();

        info!("Send Status: {}", res.status());

        match res.text() {
            Ok(txt) => debug!("Response: {}", txt),
            Err(_) => error!("Unable to get response text")
        };

        Ok(())
    }

    fn compute_signature(&self, data: &String) -> Result<String> {
        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let k = match e.decode(&self.access_key) {
            Ok(k) => k,
            Err(e) => return Err(Error::msg(e.to_string()))
        };

        let mut hmac = HMAC::new(k);

        hmac.update(data.as_bytes());
        
        let signature = hmac.finalize();

        Ok(e.encode(signature))
    }

}
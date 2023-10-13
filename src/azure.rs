use std::collections::HashMap;
use log::{debug, info};
use url::{Url, Position};
use serde::Serialize;
use base64::{Engine, alphabet};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use hmac_sha256::{Hash, HMAC};
use chrono::Utc;
use reqwest::blocking::Client;

#[derive(Serialize)]
pub struct AzureMailAddress {
    pub address: String,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>
}

#[derive(Serialize)]
pub struct AzureMailRecipients {
    pub to: Vec<AzureMailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<AzureMailAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<AzureMailAddress>>
}

#[derive(Serialize)]
pub struct AzureMailContent {
    pub subject: String,
    #[serde(rename = "plainText", skip_serializing_if = "Option::is_none")]
    pub plain_text: Option<String>,
    pub html: String
}

#[derive(Serialize)]
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

    pub fn new_with_display_name(address: String, display_name: String) -> Self {
        Self {
            address,
            display_name: Option::Some(display_name)
        }
    }
}

pub struct AzureMailClient {
    endpoint: String,
    access_key: String
}

impl AzureMailClient {

    const RFC1123: &str = "%a, %d %b %Y %H:%M:%S GMT";
    
    pub fn new(endpoint: &String, access_key: &String) -> Self {
        Self {
            endpoint: endpoint.clone(),
            access_key: access_key.clone()
        }
    }

    pub fn send_mail(&self, body: &AzureMailMessage) {
        let mut url = Url::parse(self.endpoint.as_str()).unwrap();

        url.set_query(Option::Some("api-version=2023-03-31"));

        let path_and_query = &url[Position::BeforePath..];
        let host = url.host_str().unwrap();
        let body = serde_json::to_string(body).unwrap();

        info!("URL: {}", url);
        info!("Content: {}", body);

        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let mut hash = Hash::new();
        
        hash.update(body.as_bytes());

        let content_hash = e.encode(hash.finalize());
        let date = Utc::now().format(AzureMailClient::RFC1123).to_string();
        let str_to_sign = String::from(format!("POST\n{}\n{};{};{}", path_and_query, date, host, content_hash));
        let signature = self.compute_signature(str_to_sign.as_str());
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
            .send()
            .unwrap();

        info!("Send Status: {}", res.status());
        debug!("Response: {}", res.text().unwrap());
    }

    fn compute_signature(&self, data: &str) -> String {
        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let k = e.decode(self.access_key.clone()).unwrap();

        let mut hmac = HMAC::new(k);
        let data = data.as_bytes();

        hmac.update(data);
        
        let signature = hmac.finalize();

        e.encode(signature)
    }

}
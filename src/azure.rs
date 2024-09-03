use std::collections::HashMap;
use anyhow::{Error, Result};
use log::{debug, error, info, trace, log_enabled, Level};
use reqwest::header::{HeaderMap, HeaderValue};
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

        let date = Utc::now().format(AzureMailClient::RFC1123).to_string();

        trace!("URL: {}", url);
        trace!("Host: {}", host);
        trace!("Path & Query: {}", path_and_query);
        trace!("Body: {}", body);
        trace!("Endpoint: {}", self.endpoint);
        trace!("Access Key: {}", self.access_key);
        trace!("Date: {}", date);

        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let mut hash = Hash::new();
        
        hash.update(body.as_bytes());

        let content_hash = e.encode(hash.finalize());        
        let str_to_sign = format!("POST\n{}\n{};{};{}", path_and_query, date, host, content_hash);

        trace!("String to Sign: {}", str_to_sign);

        let signature = match self.compute_signature(&str_to_sign) {
            Ok(s) => s,
            Err(e) => return Err(Error::msg(e.to_string()))
        };
        
        let auth = format!("HMAC-SHA256 SignedHeaders=x-ms-date;host;x-ms-content-sha256&Signature={}", signature);
        let guid = uuid::Uuid::new_v4().to_string();

        let client = Client::new();
        let mut headers = HeaderMap::new();

        headers.append("Authorization", HeaderValue::from_str(&auth)?);
        headers.append("x-ms-date", HeaderValue::from_str(&date)?);
        headers.append("x-ms-content-sha256", HeaderValue::from_str(&content_hash)?);
        headers.append("Repeatability-Request-Id", HeaderValue::from_str(&guid)?);
        headers.append("Repeatability-First-Sent", HeaderValue::from_str(&date)?);
        headers.append("Content-Type", HeaderValue::from_str("application/json")?);

        if log_enabled!(Level::Trace) {
            for (key, value) in headers.iter() {
                trace!("{}: {:?}", key, value);
            }
        }

        let req = client
            .post(url.to_string())
            .headers(headers)
            .body(body);

        let res = match req.send() {
            Ok(res) => res,
            Err(e) => return Err(Error::msg(e.to_string()))
        };

        info!("Response Status: {}", res.status());

        match res.text() {
            Ok(txt) => debug!("Response Text: {}", txt),
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
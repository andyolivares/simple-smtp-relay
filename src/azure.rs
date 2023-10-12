use url::{Url, Position};
use log::debug;
use base64::{Engine, alphabet};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use hmac_sha256::{Hash, HMAC};
use chrono::Utc;

const TEST_BODY: &str = "{ \"sender\": \"gitlab@ncrfsdps.com\", \"recipients\": { \"to\": [ { \"email\": \"ao185083@ncratleos.com\" } ] }, \"content\": { \"subject\": \"Test Email\", \"plainText\": \"Hello world\" } }";

pub struct AzureMailClient {
    endpoint: String,
    access_key: String
}

impl AzureMailClient {

    const RFC1123: &str = "%a, %d %b %Y %H:%M:%S GMT";
    
    pub fn new(endpoint: String, access_key: String) -> Self {
        Self {
            endpoint,
            access_key
        }
    }

    pub fn send_mail(&self) {
        let url = Url::parse(self.endpoint.as_str()).unwrap();
        let path_and_query = &url[Position::BeforePath..];
        let host = url.host_str().unwrap();

        let e = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());
        let mut hash = Hash::new();
        
        hash.update(TEST_BODY.as_bytes());

        let content_hash = e.encode(hash.finalize());
        let date = Utc::now().format(AzureMailClient::RFC1123).to_string();
        let str_to_sign = String::from(format!("POST\n{}\n{};{};{}", path_and_query, date, host, content_hash));
        let signature = self.compute_signature(str_to_sign.as_str());

        debug!("Content hash: {}", content_hash);
        debug!("String to Sign: {}", str_to_sign);
        debug!("Signature: {}", signature);
        debug!("GUID: {}", uuid::Uuid::new_v4().to_string());
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
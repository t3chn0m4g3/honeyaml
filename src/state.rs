use std::collections::HashMap;

use jwt_simple::prelude::HS256Key;

use crate::{ logger, schema };

pub struct AppState {
    pub logger: logger::Logger,
    pub paths: Vec<schema::WebPath>,
    pub accounts: Vec<HashMap<String, String>>,
    pub key: HS256Key,
    pub jwt_issuer: String,
    pub jwt_subject: String,
    pub jwt_audience: String,
}
impl Default for AppState {
    fn default() -> AppState {
        AppState {
            logger: logger::Logger::new(),
            paths: vec![],
            accounts: vec![],
            key: HS256Key::generate(),
            jwt_issuer: "".to_owned(),
            jwt_subject: "".to_owned(),
            jwt_audience: "".to_owned(),
        }
    }
}

pub fn generate_key() -> HS256Key {
    HS256Key::generate()
}

pub fn key_from_bytes(bytes: &[u8]) -> HS256Key {
    HS256Key::from_bytes(bytes)
}
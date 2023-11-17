use std::collections::HashMap;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub api_port: u16,
    pub log_level: String,
    pub map: HashMap<String, String>
}
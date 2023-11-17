use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::{log, Level};
use serde::{Deserialize};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub log_level: String,
    pub map: BTreeMap<String, Vec<String>>
}

struct ApiRouteRoulette {
    routes: Vec<String>,
    size: usize,
    counter: AtomicUsize
}

impl ApiRouteRoulette {
    pub fn new(routes: Vec<String>) -> Self{
        ApiRouteRoulette{
            size: routes.len(),
            routes,
            counter: AtomicUsize::new(0)
        }
    }
    pub fn next(&self) -> String {
        let idx = self.counter.fetch_add(1, Ordering::AcqRel) + 1;
        let ret = &self.routes[idx % self.size];
        ret.clone()
    }
}

pub struct RedirectionReader {
    lock: RwLock<BTreeMap<String, ApiRouteRoulette>>
}

impl RedirectionReader {
    pub fn new(lock: BTreeMap<String, Vec<String>>) -> Self {
        let mut new_map: BTreeMap<String, ApiRouteRoulette> = BTreeMap::new();
        for (key, value) in lock {
            if value.len() == 0 {
                log!(Level::Warn, "| {:<15} | Route {} is ignored due to not having any remap address", "internal", &key);
                continue;
            }
            new_map.insert(key, ApiRouteRoulette::new(value));
        }
        RedirectionReader{
            lock: RwLock::new(new_map)
        }
    }
    pub async fn match_uri(&self, uri: &String) -> Option<String> {
        let locked = self.lock.read().await;
        for (key, roulette) in locked.iter() {
            if uri.starts_with(key) {
                let split = &uri[key.len()..];
                let value = roulette.next();
                return Some(value.clone() + split);
            }
        }
        None
    }
}

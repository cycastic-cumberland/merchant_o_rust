use std::collections::HashMap;
use tokio::sync::RwLock;


pub struct RedirectionReader {
    lock: RwLock<HashMap<String, String>>
}

impl RedirectionReader {
    pub fn new(lock: RwLock<HashMap<String, String>>) -> Self {
        RedirectionReader{
            lock
        }
    }
    // pub async fn read(&self, key: &String) -> Option<String> {
    //     let locked = self.lock.read().await;
    //     return match locked.get(key) {
    //         Some(v) => Some(v.clone()),
    //         None => None
    //     }
    // }
    pub async fn match_uri(&self, uri: &String) -> Option<String> {
        let locked = self.lock.read().await;
        for (key, value) in locked.iter() {
            if uri.starts_with(key) {
                let split = &uri[key.len()..];
                return Some(value.clone() + split);
            }
        }
        None
    }
}

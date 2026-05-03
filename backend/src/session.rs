use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(30 * 24 * 3600);

pub struct SessionStore(RwLock<HashMap<String, Instant>>);

impl SessionStore {
    pub fn new() -> Self {
        SessionStore(RwLock::new(HashMap::new()))
    }

    pub fn create(&self) -> String {
        let token = crate::key::random_hex_key();
        self.0.write().unwrap().insert(token.clone(), Instant::now() + TTL);
        token
    }

    pub fn is_valid(&self, token: &str) -> bool {
        self.0.read().unwrap().get(token).is_some_and(|exp| Instant::now() < *exp)
    }

    pub fn revoke(&self, token: &str) {
        self.0.write().unwrap().remove(token);
    }
}

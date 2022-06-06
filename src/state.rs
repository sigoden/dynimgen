use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::default()));
}

#[derive(Debug, Default)]
pub struct State {
    allow_urls: Vec<String>,
    fetch_timeout: u64,
}

impl State {
    pub fn set_allow_urls(&mut self, allow_urls: &[String]) -> &mut Self {
        self.allow_urls = allow_urls.to_vec();
        self
    }

    pub fn set_fetch_timeout(&mut self, timeout: u64) -> &mut Self {
        self.fetch_timeout = timeout;
        self
    }

    pub fn allow_url(&self, url: &str) -> bool {
        if self.allow_urls.is_empty() {
            return true;
        }
        self.allow_urls
            .iter()
            .any(|prefix| url.starts_with(prefix.as_str()))
    }

    pub fn fetch_timeout(&self) -> u64 {
        self.fetch_timeout
    }
}

use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::default()));
}

#[derive(Debug, Default)]
pub struct State {
    allow_urls: Vec<String>,
}

impl State {
    pub fn set_allow_urls(&mut self, allow_urls: &[String]) {
        self.allow_urls = allow_urls.to_vec();
    }
    pub fn guard_url(&self, url: &str) -> bool {
        if self.allow_urls.is_empty() {
            return true;
        }
        self.allow_urls
            .iter()
            .any(|prefix| url.starts_with(prefix.as_str()))
    }
}

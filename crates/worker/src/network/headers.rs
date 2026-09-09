use std::ops::Index;

use http::{HeaderMap, HeaderName, HeaderValue};

/// An insertion-ordered HTTP header list. Duplicate fields remain distinct.
#[derive(Debug, Clone, Default)]
pub struct HeaderList(Vec<(HeaderName, HeaderValue)>);
impl HeaderList {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn append(&mut self, name: impl TryInto<HeaderName>, value: HeaderValue) {
        if let Ok(name) = name.try_into() {
            self.0.push((name, value));
        }
    }
    pub fn insert(&mut self, name: impl TryInto<HeaderName>, value: HeaderValue) {
        if let Ok(name) = name.try_into() {
            self.0.retain(|(candidate, _)| candidate != name);
            self.0.push((name, value));
        }
    }
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.get(name).is_some()
    }
    pub fn remove(&mut self, name: impl AsRef<str>) {
        let name = name.as_ref();
        self.0
            .retain(|(candidate, _)| !candidate.as_str().eq_ignore_ascii_case(name));
    }
    pub fn get(&self, name: impl AsRef<str>) -> Option<&HeaderValue> {
        let name = name.as_ref();
        self.0
            .iter()
            .find(|(candidate, _)| candidate.as_str().eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }
    pub fn get_all(&self, name: impl AsRef<str>) -> impl Iterator<Item = &HeaderValue> {
        let name = name.as_ref().to_string();
        self.0
            .iter()
            .filter(move |(candidate, _)| candidate.as_str().eq_ignore_ascii_case(&name))
            .map(|(_, value)| value)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.0.iter().map(|(name, value)| (name, value))
    }
}
impl From<HeaderMap> for HeaderList {
    fn from(headers: HeaderMap) -> Self {
        let mut result = Self::new();
        let mut current_name = None;
        for (name, value) in headers {
            if let Some(name) = name {
                current_name = Some(name);
            }
            if let Some(name) = current_name.clone() {
                result.append(name, value);
            }
        }
        result
    }
}
impl Index<&str> for HeaderList {
    type Output = HeaderValue;
    fn index(&self, name: &str) -> &Self::Output {
        self.get(name).expect("header not found")
    }
}

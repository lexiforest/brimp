use std::sync::Arc;

use jsc::JsException;
use network::{CurlResourceLoader, ResourceLoader};

use crate::{Page, PageOptions};

pub struct Browser {
    loader: Arc<dyn ResourceLoader>,
}

impl Browser {
    pub fn new() -> Result<Self, JsException> {
        Ok(Self {
            loader: Arc::new(CurlResourceLoader::default()),
        })
    }

    pub fn with_resource_loader(loader: Arc<dyn ResourceLoader>) -> Self {
        Self { loader }
    }

    pub fn new_page(&self, options: PageOptions) -> Result<Page, JsException> {
        Page::new(options, Arc::clone(&self.loader))
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new().expect("JavaScriptCore should create a browser runtime")
    }
}

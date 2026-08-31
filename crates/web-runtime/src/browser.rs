use std::sync::Arc;

use jsc::JsException;
use network::{CurlResourceLoader, ResourceLoader};

use crate::{Page, PageOptions, page::DocumentNetworkScope, worker::WorkerCoordinator};
use web_bindings::CookieJar;

pub struct Browser {
    loader: Arc<dyn ResourceLoader>,
    workers: WorkerCoordinator,
    cookies: Arc<CookieJar>,
}

impl Browser {
    pub fn new() -> Result<Self, JsException> {
        Ok(Self {
            loader: Arc::new(CurlResourceLoader::default()),
            workers: WorkerCoordinator::new().map_err(JsException::from_message)?,
            cookies: Arc::new(CookieJar::default()),
        })
    }

    pub fn with_resource_loader(loader: Arc<dyn ResourceLoader>) -> Self {
        Self {
            loader,
            workers: WorkerCoordinator::new().expect("shared worker coordinator must start"),
            cookies: Arc::new(CookieJar::default()),
        }
    }

    pub fn new_page(&self, options: PageOptions) -> Result<Page, JsException> {
        Page::new(
            options,
            DocumentNetworkScope::new(Arc::clone(&self.loader)),
            self.workers.clone(),
            Arc::clone(&self.cookies),
        )
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new().expect("JavaScriptCore should create a browser runtime")
    }
}

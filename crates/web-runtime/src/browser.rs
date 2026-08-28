use std::sync::Arc;

use jsc::JsException;
use network::{CurlResourceLoader, ResourceLoader};

use crate::{Page, PageOptions, worker::WorkerCoordinator};

pub struct Browser {
    loader: Arc<dyn ResourceLoader>,
    workers: WorkerCoordinator,
}

impl Browser {
    pub fn new() -> Result<Self, JsException> {
        Ok(Self {
            loader: Arc::new(CurlResourceLoader::default()),
            workers: WorkerCoordinator::new().map_err(JsException::from_message)?,
        })
    }

    pub fn with_resource_loader(loader: Arc<dyn ResourceLoader>) -> Self {
        Self {
            loader,
            workers: WorkerCoordinator::new().expect("shared worker coordinator must start"),
        }
    }

    pub fn new_page(&self, options: PageOptions) -> Result<Page, JsException> {
        Page::new(options, Arc::clone(&self.loader), self.workers.clone())
    }
}

impl Default for Browser {
    fn default() -> Self {
        Self::new().expect("JavaScriptCore should create a browser runtime")
    }
}

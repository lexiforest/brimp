use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};

pub struct UnusedLoader;

#[async_trait::async_trait]
impl ResourceLoader for UnusedLoader {
    async fn fetch(&self, _: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        panic!("subsystem option tests do not navigate")
    }
}

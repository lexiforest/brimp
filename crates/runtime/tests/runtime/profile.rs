use async_trait::async_trait;
use brimp_runtime::{network::*, runtime::native::*};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct IdentityLoader(Mutex<Vec<ResourceRequest>>);
#[async_trait]
impl ResourceLoader for IdentityLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let url = request.url.clone();
        self.0.lock().unwrap().push(request);
        Ok(ResourceResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            body: b"<title>Identity</title>".to_vec(),
            effective_url: url,
            metadata: Default::default(),
        })
    }
}
#[test]
fn identity_overrides_apply_to_requests_and_each_realm() {
    let loader = Arc::new(IdentityLoader::default());
    let session = Session::with_resource_loader(
        SessionOptions {
            identity: IdentityOptions {
                user_agent: Some("Mozilla/5.0 (Custom) Custom/1 Safari/605.1.15".into()),
                languages: Some(vec!["fr-CA".into(), "fr".into()]),
                platform: Some("Custom".into()),
            },
            ..Default::default()
        },
        loader.clone(),
    )
    .unwrap();
    let page = session.new_page().unwrap();
    for agent in [
        "Mozilla/5.0 (Custom) Custom/1 Safari/605.1.15",
        "Mozilla/5.0 (Custom) Custom/2 Safari/605.1.15",
    ] {
        page.set_identity(IdentityOptions {
            user_agent: Some(agent.into()),
            ..Default::default()
        })
        .unwrap();
        page.request("GET", "https://identity.test/", Default::default())
            .unwrap();
        assert_eq!(page.evaluate("[navigator.userAgent,navigator.language,navigator.platform,typeof screen,typeof innerWidth]").unwrap(),serde_json::json!([agent,"fr-CA","Custom","object","number"]));
        let requests = loader.0.lock().unwrap();
        let request = requests.last().unwrap();
        assert_eq!(request.headers["user-agent"], agent);
        assert_eq!(request.headers["accept-language"], "fr-CA,fr;q=0.9");
    }
    assert!(
        page.set_identity(IdentityOptions {
            languages: Some(vec![]),
            ..Default::default()
        })
        .is_err()
    );
    session.close();
}

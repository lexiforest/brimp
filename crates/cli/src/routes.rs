use crate::{Result, invalid};
use brimp_runtime::runtime::{
    native::NativePage,
    routes::{RouteOverrides, RouteRegistration, RouteStage},
};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    pattern: String,
    #[serde(default = "request_stage")]
    stage: RouteStage,
    decision: Decision,
}
fn request_stage() -> RouteStage {
    RouteStage::Request
}
#[derive(Clone, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
enum Decision {
    Block {
        #[serde(default = "blocked")]
        message: String,
    },
    Fulfill {
        #[serde(default = "ok")]
        status: u16,
        #[serde(default)]
        headers: Vec<(String, String)>,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        file: Option<PathBuf>,
    },
    Continue {
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        method: Option<String>,
        #[serde(default)]
        headers: Option<Vec<(String, String)>>,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        status: Option<u16>,
    },
}
fn blocked() -> String {
    "blocked by CLI route".into()
}
fn ok() -> u16 {
    200
}
pub struct Routes(Vec<(Rule, Vec<u8>)>);
impl Routes {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let rules: Vec<Rule> = match path {
            Some(path) => {
                serde_json::from_slice(&std::fs::read(path).map_err(invalid)?).map_err(invalid)?
            }
            None => Vec::new(),
        };
        let base = path.and_then(Path::parent).unwrap_or(Path::new("."));
        let mut result = Vec::new();
        for rule in rules {
            glob::Pattern::new(&rule.pattern).map_err(invalid)?;
            if rule.stage == RouteStage::Auth {
                return Err(invalid("use --auth for origin-scoped authentication"));
            }
            let mut bytes = Vec::new();
            match &rule.decision {
                Decision::Block { .. } => {}
                Decision::Fulfill {
                    status,
                    headers,
                    body,
                    file,
                } => {
                    http::StatusCode::from_u16(*status).map_err(invalid)?;
                    validate_headers(headers)?;
                    if body.is_some() && file.is_some() {
                        return Err(invalid("route body and file conflict"));
                    }
                    bytes = if let Some(file) = file {
                        std::fs::read(base.join(file)).map_err(invalid)?
                    } else {
                        body.clone().unwrap_or_default().into_bytes()
                    };
                }
                Decision::Continue {
                    url,
                    method,
                    headers,
                    body,
                    status,
                } => {
                    if let Some(url) = url {
                        url::Url::parse(url).map_err(invalid)?;
                    }
                    if let Some(method) = method {
                        http::Method::from_bytes(method.as_bytes()).map_err(invalid)?;
                    }
                    if let Some(headers) = headers {
                        validate_headers(headers)?;
                    }
                    if let Some(status) = status {
                        http::StatusCode::from_u16(*status).map_err(invalid)?;
                    }
                    if rule.stage == RouteStage::Request && status.is_some() {
                        return Err(invalid("request route cannot set status"));
                    }
                    if rule.stage == RouteStage::Response && (url.is_some() || method.is_some()) {
                        return Err(invalid("response route cannot set URL/method"));
                    }
                    if let Some(body) = body {
                        bytes = body.as_bytes().to_vec();
                    }
                }
            }
            if bytes.len() > brimp_runtime::runtime::routes::MAX_INTERCEPTED_BODY {
                return Err(invalid("route body exceeds interception limit"));
            }
            result.push((rule, bytes));
        }
        Ok(Self(result))
    }
    pub fn install(
        self,
        page: &NativePage,
        errors: Arc<Mutex<Option<String>>>,
    ) -> Result<Vec<RouteRegistration>> {
        self.0
            .into_iter()
            .map(|(rule, bytes)| {
                let errors = errors.clone();
                page.route(
                    &rule.pattern,
                    rule.stage,
                    Arc::new(move |route| {
                        let result = match &rule.decision {
                            Decision::Block { message } => route.fail(message),
                            Decision::Fulfill {
                                status, headers, ..
                            } => route.fulfill(*status, headers.clone(), bytes.clone()),
                            Decision::Continue {
                                url,
                                method,
                                headers,
                                body,
                                status,
                            } => route.continue_request(RouteOverrides {
                                url: url.clone(),
                                method: method.clone(),
                                headers: headers.clone(),
                                body: body.as_ref().map(|_| bytes.clone()),
                                status: *status,
                            }),
                        };
                        if let Err(error) = result {
                            *errors.lock().unwrap() = Some(error.to_string());
                            if route.is_pending() {
                                let _ = route.fail("invalid route decision");
                            }
                        }
                    }),
                )
            })
            .collect()
    }
}
fn validate_headers(headers: &[(String, String)]) -> Result<()> {
    for (name, value) in headers {
        http::HeaderName::from_bytes(name.as_bytes()).map_err(invalid)?;
        http::HeaderValue::from_str(value).map_err(invalid)?;
    }
    Ok(())
}

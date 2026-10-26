//! Page script classification, scheduling, and module loading.

mod compiler;

use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    rc::Rc,
    sync::Arc,
};

use super::{NavigationError, Page};
use crate::dom::{HtmlParserSession, NodeId, ParseProgress};
use crate::network::NetworkError;
use brimp_jsc::JsException;

impl Page {
    pub(super) async fn parse_navigation_document(
        &mut self,
        html: &str,
        document_url: &str,
    ) -> Result<(), NavigationError> {
        let base_url = url::Url::parse(document_url)?;
        let mut parser = HtmlParserSession::new(Rc::clone(&self.document), html)
            .map_err(|error| NavigationError::Document(error.to_string()))?;
        let mut script_order = 0;
        let mut deferred_order = Vec::new();
        let mut pending_loads = tokio::task::JoinSet::new();
        let mut loaded_deferred = BTreeMap::new();
        let mut module_scripts = Vec::new();

        loop {
            let progress = parser
                .resume()
                .map_err(|error| NavigationError::Document(error.to_string()))?;
            let mutations = parser
                .take_mutations()
                .map_err(|error| NavigationError::Document(error.to_string()))?;
            self.bindings
                .deliver_parser_mutations(&self.js, &mutations)?;
            let ParseProgress::Script(node_id) = progress else {
                break;
            };
            let order = script_order;
            script_order += 1;
            let Some(script) = self.script_from_node(node_id, order) else {
                continue;
            };
            if script.kind == ScriptKind::Module {
                module_scripts.push(script);
                continue;
            }
            match (script.mode, script.source) {
                (ScriptMode::Blocking, ScriptSource::Inline(source)) => {
                    self.execute_document_script(&source, script.order);
                }
                (ScriptMode::Blocking, ScriptSource::External(src)) => {
                    let result = async {
                        let script_url = base_url.join(&src)?;
                        let response = self.fetch_success(script_url.as_str()).await?;
                        Ok::<_, NavigationError>(String::from_utf8(response.body)?)
                    }
                    .await;
                    match result {
                        Ok(source) => self.execute_document_script(&source, script.order),
                        Err(error) => self.report_script_load_error(&src, &error),
                    }
                }
                (ScriptMode::Async, ScriptSource::External(src)) => {
                    let script_url = base_url.join(&src)?.to_string();
                    let request = self.resource_request(&script_url)?;
                    let sender = self.tasks.sender();
                    let realm = self.realm_id;
                    self.pending_navigation_scripts
                        .set(self.pending_navigation_scripts.get() + 1);
                    let loader = Arc::clone(&self.network_scope.loader);
                    let context = Arc::clone(&self.browsing_context);
                    let job = self.network_scope.executor.spawn(async move {
                        let result =
                            crate::runtime::request::fetch(loader.as_ref(), &context, request)
                                .await;
                        let _ = sender.post(move |page| {
                            if page.realm_id != realm {
                                return;
                            }
                            let loaded = LoadedScript {
                                order: script.order,
                                mode: ScriptMode::Async,
                                source: script_url,
                                result,
                            };
                            let _ = page.handle_loaded_script(loaded, &mut BTreeMap::new());
                            page.pending_navigation_scripts
                                .set(page.pending_navigation_scripts.get().saturating_sub(1));
                            if page.document_ready && page.pending_navigation_scripts.get() == 0 {
                                let _ = page.finish_navigation_load();
                            }
                        });
                    });
                    self.navigation_jobs.borrow_mut().push(job.abort_handle());
                }
                (mode, ScriptSource::External(src)) => {
                    if mode == ScriptMode::Defer {
                        deferred_order.push(script.order);
                    }
                    let script_url = match base_url.join(&src) {
                        Ok(url) => url.to_string(),
                        Err(error) => {
                            self.report_script_load_error(&src, &error.into());
                            continue;
                        }
                    };
                    let request = self.resource_request(&script_url)?;
                    let source = script_url.clone();
                    let loader = Arc::clone(&self.network_scope.loader);
                    let browsing_context = Arc::clone(&self.browsing_context);
                    pending_loads.spawn(async move {
                        let result = crate::runtime::request::fetch(
                            loader.as_ref(),
                            &browsing_context,
                            request,
                        )
                        .await;
                        LoadedScript {
                            order: script.order,
                            mode,
                            source,
                            result,
                        }
                    });
                }
                (_, ScriptSource::Inline(_)) => unreachable!("inline scripts are blocking"),
            }

            while let Some(result) = pending_loads.try_join_next() {
                self.handle_loaded_script(result?, &mut loaded_deferred)?;
            }
            self.run_pending_tasks()?;
        }

        self.bindings.script_control(
            &self.js,
            serde_json::json!({"op":"ready", "state":"interactive"}),
        )?;
        self.run_pending_tasks()?;
        let mut next_deferred = 0;
        self.execute_ready_deferred(&deferred_order, &mut next_deferred, &mut loaded_deferred)?;
        while let Some(result) = pending_loads.join_next().await {
            self.run_pending_tasks()?;
            self.handle_loaded_script(result?, &mut loaded_deferred)?;
            self.execute_ready_deferred(&deferred_order, &mut next_deferred, &mut loaded_deferred)?;
        }
        for script in module_scripts {
            let result = match script.source {
                ScriptSource::Inline(source) => {
                    let url = format!("{document_url}#inline-module-{}", script.order);
                    self.load_and_execute_module(url, source).await
                }
                ScriptSource::External(source) => {
                    let url = match base_url.join(&source) {
                        Ok(url) => url.to_string(),
                        Err(error) => {
                            self.report_script_load_error(&source, &error.into());
                            continue;
                        }
                    };
                    match self.fetch_success(&url).await {
                        Ok(response) => match String::from_utf8(response.body) {
                            Ok(body) => self.load_and_execute_module(url, body).await,
                            Err(error) => Err(error.to_string()),
                        },
                        Err(error) => Err(error.to_string()),
                    }
                }
            };
            if let Err(error) = result {
                self.report_page_script_error(&JsException::from_message(error));
            }
        }
        self.process_dynamic_scripts().await;
        Ok(())
    }

    fn script_from_node(&self, node_id: NodeId, order: usize) -> Option<Script> {
        let document = self.document.borrow();
        let script_type = document
            .attribute(node_id, "type")
            .unwrap_or_default()
            .to_string_lossy()
            .trim()
            .to_ascii_lowercase();
        let kind = match script_type.as_str() {
            "" | "text/javascript" | "application/javascript" => ScriptKind::Classic,
            "module" => ScriptKind::Module,
            _ => return None,
        };
        let source = match document.attribute(node_id, "src") {
            Some(src) => ScriptSource::External(src.to_string_lossy().into_owned()),
            None => {
                ScriptSource::Inline(document.text(node_id).ok()?.to_string_lossy().into_owned())
            }
        };
        let mode = match (kind, &source) {
            (ScriptKind::Module, _) if document.attribute(node_id, "async").is_some() => {
                ScriptMode::Async
            }
            (ScriptKind::Module, _) => ScriptMode::Defer,
            (ScriptKind::Classic, ScriptSource::Inline(_)) => ScriptMode::Blocking,
            (ScriptKind::Classic, ScriptSource::External(_))
                if document.attribute(node_id, "async").is_some() =>
            {
                ScriptMode::Async
            }
            (ScriptKind::Classic, ScriptSource::External(_))
                if document.attribute(node_id, "defer").is_some() =>
            {
                ScriptMode::Defer
            }
            (ScriptKind::Classic, ScriptSource::External(_)) => ScriptMode::Blocking,
        };
        Some(Script {
            order,
            source,
            mode,
            kind,
        })
    }

    async fn load_and_execute_module(
        &self,
        root_url: String,
        root_source: String,
    ) -> Result<(), String> {
        let mut pending = VecDeque::from([(root_url.clone(), root_source)]);
        let mut seen = HashSet::new();
        while let Some((module_url, source)) = pending.pop_front() {
            if !seen.insert(module_url.clone()) {
                continue;
            }
            let compiled = compiler::compile(&source, &module_url)?;
            for specifier in &compiled.dependencies {
                let dependency_url = resolve_module_specifier(&module_url, specifier)?;
                if seen.contains(&dependency_url) {
                    continue;
                }
                let response = self
                    .fetch_success(&dependency_url)
                    .await
                    .map_err(|error| error.to_string())?;
                let source = String::from_utf8(response.body).map_err(|error| error.to_string())?;
                pending.push_back((dependency_url, source));
            }
            self.js
                .call_function_with_string(
                    &self.module_loader,
                    &serde_json::json!({
                        "op": "register", "url": module_url, "code": compiled.code,
                    })
                    .to_string(),
                )
                .map_err(|error| error.message().to_owned())?;
        }

        let _ = self.bindings.sync_window_named_properties(&self.js);
        if let Err(error) = self.js.call_function_with_string(
            &self.module_loader,
            &serde_json::json!({
                "op": "evaluate", "url": root_url,
            })
            .to_string(),
        ) {
            self.report_page_script_error(&error);
        }
        self.finish_script_execution();
        Ok(())
    }

    pub(crate) async fn process_dynamic_scripts(&self) {
        loop {
            let serialized = match self
                .bindings
                .script_control(&self.js, serde_json::json!({"op":"take"}))
            {
                Ok(value) => value,
                Err(error) => {
                    self.report_page_script_error(&error);
                    return;
                }
            };
            let pending: Vec<DynamicScript> = match serde_json::from_str(&serialized) {
                Ok(pending) => pending,
                Err(error) => {
                    self.report_page_script_error(&JsException::from_message(error.to_string()));
                    return;
                }
            };
            if pending.is_empty() {
                return;
            }
            for script in pending {
                let result = match (script.module, script.src) {
                    (true, Some(url)) => match self.fetch_success(&url).await {
                        Ok(response) => match String::from_utf8(response.body) {
                            Ok(source) => self.load_and_execute_module(url, source).await,
                            Err(error) => Err(error.to_string()),
                        },
                        Err(error) => Err(error.to_string()),
                    },
                    (true, None) => {
                        let base = self
                            .browsing_context
                            .current_url()
                            .unwrap_or_else(|| "about:blank".to_owned());
                        self.load_and_execute_module(
                            format!("{base}#dynamic-module-{}", script.id),
                            script.source,
                        )
                        .await
                    }
                    (false, Some(url)) => match self.fetch_success(&url).await {
                        Ok(response) => String::from_utf8(response.body)
                            .map(|source| self.execute_dynamic_script(&source, script.id))
                            .map_err(|error| error.to_string()),
                        Err(error) => Err(error.to_string()),
                    },
                    (false, None) => {
                        self.execute_dynamic_script(&script.source, script.id);
                        Ok(())
                    }
                };
                if let Err(error) = &result {
                    self.report_page_script_error(&JsException::from_message(error.clone()));
                }
                if let Err(error) = self.bindings.script_control(
                    &self.js,
                    serde_json::json!({
                        "op": "complete", "id": script.id, "error": result.is_err(),
                    }),
                ) {
                    self.report_page_script_error(&error);
                }
                self.finish_script_execution();
            }
        }
    }

    fn handle_loaded_script(
        &self,
        loaded: LoadedScript,
        deferred: &mut BTreeMap<usize, String>,
    ) -> Result<(), NavigationError> {
        let response = match loaded.result {
            Ok(response) => match self.accept_success_response(response) {
                Ok(response) => response,
                Err(error) => {
                    self.report_script_load_error(&loaded.source, &error);
                    return Ok(());
                }
            },
            Err(error) => {
                self.report_script_load_error(&loaded.source, &error.into());
                return Ok(());
            }
        };
        let source = match String::from_utf8(response.body) {
            Ok(source) => source,
            Err(error) => {
                self.report_script_load_error(&loaded.source, &error.into());
                return Ok(());
            }
        };
        match loaded.mode {
            ScriptMode::Async => {
                self.execute_document_script(&source, loaded.order);
            }
            ScriptMode::Defer => {
                deferred.insert(loaded.order, source);
            }
            ScriptMode::Blocking => unreachable!("blocking scripts are not spawned"),
        }
        Ok(())
    }

    fn execute_ready_deferred(
        &self,
        order: &[usize],
        next: &mut usize,
        loaded: &mut BTreeMap<usize, String>,
    ) -> Result<(), NavigationError> {
        while let Some(script_order) = order.get(*next)
            && let Some(source) = loaded.remove(script_order)
        {
            self.execute_document_script(&source, *script_order);
            *next += 1;
        }
        Ok(())
    }

    pub(super) fn execute_page_script(&self, source: &str) {
        self.execute_page_script_with_current(source, None);
    }

    fn execute_document_script(&self, source: &str, index: usize) {
        self.execute_page_script_with_current(
            source,
            Some(serde_json::json!({"op":"document", "id":index})),
        );
    }

    fn execute_dynamic_script(&self, source: &str, id: u64) {
        self.execute_page_script_with_current(
            source,
            Some(serde_json::json!({"op":"dynamic", "id":id})),
        );
    }

    fn execute_page_script_with_current(&self, source: &str, current: Option<serde_json::Value>) {
        let _ = self.bindings.sync_window_named_properties(&self.js);
        if let Some(current) = current.as_ref()
            && let Err(error) = self.bindings.script_control(&self.js, current.clone())
        {
            self.report_page_script_error(&error);
        }
        let result = self.js.eval(source);
        if current.is_some()
            && let Err(error) = self
                .bindings
                .script_control(&self.js, serde_json::json!({"op":"clear"}))
        {
            self.report_page_script_error(&error);
        }
        if let Err(error) = result {
            self.report_page_script_error(&error);
        }
        self.finish_script_execution();
    }

    fn finish_script_execution(&self) {
        if let Err(error) = self.start_pending_fetches() {
            self.report_page_script_error(&error);
        }
    }

    fn report_page_script_error(&self, error: &JsException) {
        self.events.emit(crate::runtime::events::PageEvent::Error {
            message: error.to_string(),
        });
        let Ok(message) = serde_json::to_string(error.message()) else {
            return;
        };
        let _ = self.js.eval(&format!("console.error({message})"));
    }

    fn report_script_load_error(&self, source: &str, error: &NavigationError) {
        self.report_page_script_error(&JsException::from_message(format!(
            "Failed to load script {source:?}: {error}"
        )));
    }
}

enum ScriptSource {
    Inline(String),
    External(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptMode {
    Blocking,
    Defer,
    Async,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptKind {
    Classic,
    Module,
}

struct Script {
    order: usize,
    source: ScriptSource,
    mode: ScriptMode,
    kind: ScriptKind,
}

struct LoadedScript {
    order: usize,
    mode: ScriptMode,
    source: String,
    result: Result<crate::network::ResourceResponse, NetworkError>,
}

#[derive(serde::Deserialize)]
struct DynamicScript {
    id: u64,
    module: bool,
    src: Option<String>,
    source: String,
}

fn resolve_module_specifier(base: &str, specifier: &str) -> Result<String, String> {
    if let Ok(url) = url::Url::parse(specifier) {
        return Ok(url.to_string());
    }
    if !(specifier.starts_with('/') || specifier.starts_with("./") || specifier.starts_with("../"))
    {
        return Err(format!(
            "bare module specifier {specifier:?} requires an import map"
        ));
    }
    url::Url::parse(base)
        .and_then(|base| base.join(specifier))
        .map(|url| url.to_string())
        .map_err(|error| format!("could not resolve module {specifier:?} from {base}: {error}"))
}

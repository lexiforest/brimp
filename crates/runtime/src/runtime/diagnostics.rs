//! Bounded diagnostic records. No page objects or property getters are serialized.
use brimp_jsc::JsException;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    fs::{File, OpenOptions},
    io::Write,
    rc::Rc,
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceMode {
    #[default]
    Api,
    Recursive,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticsOptions {
    pub mode: TraceMode,
    pub path: Option<String>,
    /// Path prefixes; empty means every instrumented operation.
    pub include: Vec<String>,
    pub capacity: usize,
    pub stack: bool,
}
impl Default for DiagnosticsOptions {
    fn default() -> Self {
        Self {
            mode: TraceMode::Api,
            path: None,
            include: Vec::new(),
            capacity: 4096,
            stack: false,
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceRecord {
    pub sequence: u64,
    pub page_id: u64,
    pub realm_id: u64,
    pub elapsed_ms: f64,
    pub kind: String,
    pub path: String,
    pub outcome: String,
    pub stack: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Clone, Copy)]
pub(crate) enum DiagnosticsView {
    Take,
    Peek,
    Summary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Coverage {
    enabled: bool,
    mode: TraceMode,
    include: Vec<String>,
    limitations: &'static [&'static str],
}
impl Coverage {
    fn new(options: Option<&DiagnosticsOptions>) -> Self {
        Self {
            enabled: options.is_some(),
            mode: options.map(|o| o.mode).unwrap_or_default(),
            include: options.map(|o| o.include.clone()).unwrap_or_default(),
            limitations: if options.is_some_and(|o| o.mode == TraceMode::Recursive) {
                &[
                    "Recursive environment proxies, not engine-wide or lexical-variable instrumentation",
                    "Raw aliases, internal API implementation accesses and bypasses of traced roots are not observed",
                    "Invariant-constrained references and binary buffers remain raw and emit coverage notes",
                    "Proxy identity, global descriptors, reflection, timing and stacks differ from untraced execution",
                    "Thrown operations are synchronous failures, not proof of an unsupported API",
                ]
            } else {
                &[
                    "API instrumentation, not every JavaScript property operation",
                    "Plain and missing properties, direct in/delete and engine-internal accesses are not comprehensively observed",
                    "Only selected reflection operations are traced",
                    "Arbitrary namespace objects, symbol members, non-configurable descriptors and APIs installed after initialization are not comprehensively observed",
                    "Instrumentation can change function identity, reflection, timing and stacks",
                    "Thrown operations are not necessarily unsupported APIs; successful operations may be mocks",
                ]
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginScope {
    pub name: String,
    pub globals: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticBatch<'a> {
    records: &'a VecDeque<TraceRecord>,
    dropped: u64,
    write_error: Option<&'a str>,
    coverage: Coverage,
    #[serde(skip)]
    owners: &'a BTreeMap<String, String>,
}

// An API wrapper and its native dispatch are separate observations, not two
// browser dependencies. Reflection probes retain their target and member.
#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct Dependency<'a> {
    layer: &'static str,
    path: &'a str,
    kind: &'a str,
    target: Option<&'a str>,
    property: Option<&'a str>,
}
#[derive(Default, Serialize)]
struct Counts {
    count: usize,
    throws: usize,
}
#[derive(Serialize)]
struct DependencyUsage<'a> {
    #[serde(flatten)]
    dependency: Dependency<'a>,
    #[serde(flatten)]
    counts: Counts,
}
impl DiagnosticBatch<'_> {
    fn value(self, view: DiagnosticsView) -> serde_json::Value {
        if !matches!(view, DiagnosticsView::Summary) {
            return serde_json::to_value(self).expect("finite diagnostic data");
        }
        let mut dependencies = BTreeMap::<Dependency, Counts>::new();
        let mut reads =
            BTreeMap::<(String, String, Option<String>, Option<String>, bool), usize>::new();
        let mut operations = BTreeMap::<(String, String, String, Option<String>), usize>::new();
        for record in self.records {
            let recursive = is_recursive(record);
            let layer = record_layer(record);
            if recursive && record.kind == "get" && record.outcome == "undefined" {
                let details = record.details.as_ref();
                let present = details
                    .and_then(|d| d.get("present"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let target = details
                    .and_then(|d| d.get("target"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                let property = details
                    .and_then(|d| d.get("property"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                *reads
                    .entry((
                        layer.to_owned(),
                        record.path.clone(),
                        target,
                        property,
                        present,
                    ))
                    .or_default() += 1;
            } else if record.outcome == "throw" {
                let failure = record
                    .details
                    .as_ref()
                    .and_then(|d| d.get("failure"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                *operations
                    .entry((
                        layer.to_owned(),
                        record.path.clone(),
                        record.kind.clone(),
                        failure,
                    ))
                    .or_default() += 1;
            }
            let detail = |key| {
                if record.kind != "reflection" && !recursive {
                    return None;
                }
                record.details.as_ref()?.get(key)?.as_str()
            };
            let dependency = Dependency {
                layer,
                path: &record.path,
                kind: &record.kind,
                target: detail("target"),
                property: detail("property"),
            };
            let counts = dependencies.entry(dependency).or_default();
            counts.count += 1;
            counts.throws += usize::from(record.outcome == "throw");
        }
        let dependencies: Vec<_> = dependencies
            .into_iter()
            .map(|(dependency, counts)| DependencyUsage { dependency, counts })
            .collect();
        let reads: Vec<_> = reads
            .into_iter()
            .map(|((layer, path, target, property, present), count)| {
                serde_json::json!({
                    "layer": layer,
                    "path": path,
                    "target": target,
                    "property": property,
                    "present": present,
                    "count": count,
                    "owner": owner_for(self.owners, &path),
                })
            })
            .collect();
        let operations: Vec<_> = operations
            .into_iter()
            .map(|((layer, path, kind, failure), count)| {
                serde_json::json!({
                    "layer": layer,
                    "path": path,
                    "kind": kind,
                    "failure": failure,
                    "count": count,
                    "owner": owner_for(self.owners, &path),
                })
            })
            .collect();
        serde_json::json!({
            "dependencies": dependencies,
            "unresolved": {
                "reads": reads,
                "operations": operations,
                "note": "Unresolved reads are retained observations whose result was undefined; failed operations are retained synchronous throws. A trace miss is not proof that a page never touched an API, and ownership names the nearest installed surface rather than a required fix location.",
            },
            "retained": self.records.len(),
            "dropped": self.dropped,
            "writeError": self.write_error,
            "coverage": self.coverage,
        })
    }
}

fn is_recursive(record: &TraceRecord) -> bool {
    record
        .details
        .as_ref()
        .and_then(|d| d.get("layer"))
        .and_then(|v| v.as_str())
        == Some("recursive")
}

fn record_layer(record: &TraceRecord) -> &'static str {
    if is_recursive(record) {
        "recursive"
    } else {
        match record.kind.as_str() {
            "native" => "native",
            "reflection" => "reflection",
            _ => "api",
        }
    }
}

fn owner_for(owners: &BTreeMap<String, String>, path: &str) -> String {
    let path = path.strip_prefix("window.").unwrap_or(path);
    let root = path.split('.').next().unwrap_or(path);
    owners
        .get(root)
        .cloned()
        .unwrap_or_else(|| "unknown".into())
}

pub(crate) fn disabled(view: DiagnosticsView) -> serde_json::Value {
    let owners = BTreeMap::new();
    DiagnosticBatch {
        records: &VecDeque::new(),
        dropped: 0,
        write_error: None,
        coverage: Coverage::new(None),
        owners: &owners,
    }
    .value(view)
}
#[derive(Clone)]
pub(crate) struct Diagnostics(Rc<RefCell<State>>);
struct State {
    options: DiagnosticsOptions,
    file: Option<File>,
    records: VecDeque<TraceRecord>,
    sequence: u64,
    dropped: u64,
    write_error: Option<String>,
    page_id: u64,
    realm_id: u64,
    start: Instant,
    owners: BTreeMap<String, String>,
}
impl Diagnostics {
    pub fn new(options: DiagnosticsOptions, page_id: u64) -> Result<Self, JsException> {
        if options.capacity == 0 || options.capacity > 1_000_000 {
            return Err(JsException::from_message(
                "diagnostic capacity must be 1..1000000",
            ));
        }
        let file = options
            .path
            .as_ref()
            .map(|path| OpenOptions::new().create(true).append(true).open(path))
            .transpose()
            .map_err(|e| JsException::from_message(format!("diagnostic log: {e}")))?;
        Ok(Self(Rc::new(RefCell::new(State {
            options,
            file,
            records: VecDeque::new(),
            sequence: 0,
            dropped: 0,
            write_error: None,
            page_id,
            realm_id: page_id,
            start: Instant::now(),
            owners: BTreeMap::new(),
        }))))
    }
    pub fn set_owners(&self, owners: BTreeMap<String, String>) {
        self.0.borrow_mut().owners = owners;
    }
    pub fn realm(&self, id: u64) {
        self.0.borrow_mut().realm_id = id;
    }
    pub fn stack(&self) -> bool {
        self.0.borrow().options.stack
    }
    pub fn mode(&self) -> TraceMode {
        self.0.borrow().options.mode
    }
    pub fn record(
        &self,
        kind: &str,
        path: &str,
        outcome: &str,
        stack: Option<String>,
        details: Option<serde_json::Value>,
    ) {
        let mut state = self.0.borrow_mut();
        if !state.options.include.is_empty()
            && !state
                .options
                .include
                .iter()
                .any(|prefix| path.starts_with(prefix))
        {
            return;
        }
        state.sequence += 1;
        let record = TraceRecord {
            sequence: state.sequence,
            page_id: state.page_id,
            realm_id: state.realm_id,
            elapsed_ms: state.start.elapsed().as_secs_f64() * 1000.0,
            kind: kind.into(),
            path: path.chars().take(512).collect(),
            outcome: outcome.chars().take(256).collect(),
            details,
            stack: stack.map(|s| s.chars().take(4096).collect()),
        };
        if let Some(file) = &mut state.file {
            // Serialize complete lines across page owners, including shared output paths.
            static WRITER: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let _guard = WRITER.lock().unwrap_or_else(|e| e.into_inner());
            let mut line = serde_json::to_vec(&record).expect("finite trace data");
            line.push(b'\n');
            if let Err(error) = file.write_all(&line) {
                state.write_error = Some(error.to_string());
                state.file = None;
            }
        }
        if state.records.len() == state.options.capacity {
            state.records.pop_front();
            state.dropped += 1;
        }
        state.records.push_back(record);
    }
    pub fn read(&self, view: DiagnosticsView) -> serde_json::Value {
        let mut state = self.0.borrow_mut();
        let value = DiagnosticBatch {
            records: &state.records,
            dropped: state.dropped,
            write_error: state.write_error.as_deref(),
            coverage: Coverage::new(Some(&state.options)),
            owners: &state.owners,
        }
        .value(view);
        if matches!(view, DiagnosticsView::Take) {
            state.records.clear();
            state.dropped = 0;
        }
        value
    }
}

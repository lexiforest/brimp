use std::sync::{Arc, Mutex};

use brimp_runtime::{
    network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse},
    runtime::{
        events::PageEvent,
        native::{RequestOptions, Session, SessionOptions},
        plugins::{Plugin, bundled},
    },
};
use serde_json::json;

const AUDIT: &str = r#"(() => {
    const forbidden = ['process','global','Buffer','require','module','__filename','__dirname'];
    const keys = Reflect.ownKeys(globalThis).map(String);
    return [
        forbidden.filter(name => name in globalThis),
        keys.filter(name => name.startsWith('__brimp') || name === '__core-js_shared__'),
        Function('return typeof process')(),
        Object.getOwnPropertySymbols(globalThis).map(String).filter(name => /brimp|core-js/.test(name))
    ];
})()"#;

struct Fixtures;
#[async_trait::async_trait]
impl ResourceLoader for Fixtures {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let body = match url::Url::parse(&request.url).unwrap().path() {
            "/module.js" => {
                "export const answer = 42; globalThis.moduleClean = !Reflect.ownKeys(globalThis).some(k => String(k).startsWith('__brimpModule'));"
            }
            "/dynamic.js" => {
                "globalThis.dynamicCurrent = document.currentScript.src.endsWith('/dynamic.js');"
            }
            _ => {
                r#"<script>
                globalThis.firstScriptClean = !Reflect.ownKeys(globalThis).some(k => String(k).startsWith('__brimp'));
                globalThis.currentCorrect = document.currentScript === document.scripts[0];
                const script = document.createElement('script');
                script.src = '/dynamic.js';
                script.onload = () => { globalThis.dynamicLoaded = true; };
                document.head.appendChild(script);
                globalThis.dynamicElementClean = !Reflect.ownKeys(script).some(k => String(k).startsWith('__'));
            </script><script type="module">import {answer} from './module.js'; globalThis.answer = answer;</script>"#
            }
        };
        Ok(ResourceResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            body: body.as_bytes().to_vec(),
            effective_url: request.url,
            metadata: Default::default(),
        })
    }
}

#[test]
fn internal_controls_and_dependency_store_stay_private_across_navigation() {
    for plugins in [vec![], vec![bundled::storage()]] {
        let session = Session::with_resource_loader(
            SessionOptions {
                plugins,
                ..Default::default()
            },
            Arc::new(Fixtures),
        )
        .unwrap();
        let page = session.new_page().unwrap();
        assert_eq!(
            page.evaluate(AUDIT).unwrap(),
            json!([[], [], "undefined", []])
        );
        for _ in 0..2 {
            page.request("GET", "https://exposure.test/", RequestOptions::default())
                .unwrap();
            assert_eq!(
                page.evaluate(AUDIT).unwrap(),
                json!([[], [], "undefined", []])
            );
            assert_eq!(page.evaluate("[firstScriptClean,currentCorrect,moduleClean,answer,dynamicCurrent,dynamicLoaded,dynamicElementClean,document.currentScript===null]").unwrap(), json!([true,true,true,42,true,true,true,true]));
            assert_eq!(page.evaluate("const value={}; value.self=value; const copy=structuredClone(value); copy!==value && copy.self===copy").unwrap(), true);
            // No runtime sweep removes arbitrary page globals or freezes APIs.
            assert_eq!(page.evaluate("globalThis.__brimpApplicationData=42; document.applicationData=7; [__brimpApplicationData,document.applicationData]").unwrap(), json!([42,7]));
        }
    }
}

#[test]
fn page_shadowing_cannot_replace_script_controls_or_module_compiler() {
    let session = Session::with_resource_loader(Default::default(), Arc::new(Fixtures)).unwrap();
    let page = session.new_page().unwrap();
    page.request("GET", "https://exposure.test/", RequestOptions::default())
        .unwrap();
    page.evaluate(r#"
        for (const name of ['__brimpTakeDynamicScripts','__brimpCompleteDynamicScript','__brimpSetCurrentDynamicScript','__brimpClearCurrentScript']) {
            globalThis[name] = () => { throw new Error('page control called'); };
        }
        globalThis.__brimpModuleLoader = {register() { throw new Error('page loader called'); }};
        globalThis.Function = () => { throw new Error('page compiler called'); };
        const script = document.createElement('script');
        script.type = 'module';
        script.textContent = 'globalThis.privateModuleWorked = 42';
        document.head.appendChild(script);
        null;
    "#).unwrap();
    assert_eq!(page.evaluate("privateModuleWorked").unwrap(), 42);
}

#[test]
fn plugin_apis_share_source_boundary_without_invoking_getters_or_freezing_pages() {
    let plugin = Plugin::new(
        "example",
        r#"({global}) => {
        global.api = {method() { return 42; }, get answer() { return 42; }};
        global.getterCalls = 0;
        Object.defineProperty(global, 'lazyAPI', {configurable:true, get() { getterCalls++; throw new Error('do not execute during finalization'); }});
        global.failInPlugin = () => { throw new Error('public detail'); };
    }
    //# sourceURL=file:///private/host/project/secret-plugin.js
    "#,
    );
    let session = Session::new(SessionOptions {
        plugins: vec![bundled::storage(), plugin],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(
        page.evaluate(AUDIT).unwrap(),
        json!([[], [], "undefined", []])
    );
    assert_eq!(page.evaluate("[getterCalls,api.method(),api.answer,/\\[native code\\]/.test(api.method.toString()),/\\[native code\\]/.test(Storage.prototype.getItem.toString()),/\\[native code\\]/.test(Object.getOwnPropertyDescriptor(api,'answer').get.toString())]").unwrap(), json!([0,42,42,true,true,true]));
    let stack = page
        .evaluate("try { failInPlugin() } catch(e) { e.stack }")
        .unwrap();
    assert!(
        !stack.as_str().unwrap().contains("secret-plugin"),
        "{stack}"
    );
    assert!(!stack.as_str().unwrap().contains("file:///"), "{stack}");
    assert_eq!(
        page.evaluate(
            "api.method = () => 7; [api.method(),api.method.toString().includes('=> 7')]"
        )
        .unwrap(),
        json!([7, true])
    );
}

#[test]
fn host_errors_are_diagnostic_only_and_installation_retains_details() {
    let plugin = Plugin::new(
        "private",
        "({global,host}) => { global.fail = () => host.call('fail'); }",
    )
    .with_handler(
        "fail",
        Arc::new(|call| call.completion.reject("secret /private/host/path.py:42")),
    );
    let session = Session::new(SessionOptions {
        plugins: vec![plugin],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let target = errors.clone();
    let _subscription = page
        .on(
            "error",
            Arc::new(move |event| {
                if let PageEvent::Error { message } = event {
                    target.lock().unwrap().push(message);
                }
            }),
        )
        .unwrap();
    assert_eq!(page.evaluate("fail().catch(e => [e.name,e.message,e instanceof DOMException,String(e.stack).includes('/private/host')])").unwrap(), json!(["OperationError","The operation failed.",true,false]));
    assert!(
        errors
            .lock()
            .unwrap()
            .iter()
            .any(|error| error.contains("secret /private/host/path.py:42"))
    );
    let plugin = Plugin::new("install", "async ({host}) => { await host.call('fail'); }")
        .with_handler(
            "fail",
            Arc::new(|call| call.completion.reject("host installation detail")),
        );
    let session = Session::new(SessionOptions {
        plugins: vec![plugin],
        ..Default::default()
    })
    .unwrap();
    let error = session
        .new_page()
        .err()
        .expect("installation fails")
        .to_string();
    assert!(error.contains("host installation detail"), "{error}");
}

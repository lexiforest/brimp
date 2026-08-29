use std::{
    net::TcpListener,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use http::StatusCode;
use network::{
    HeaderList, NetworkError, ResourceLoader, ResourceRequest, ResourceResponse,
    ResourceStreamCallback, ResourceStreamEvent, ResourceStreamHandle,
};
use web_runtime::{Browser, PageOptions, PersistentStorageOptions};

#[path = "subsystem_options/audio.rs"]
mod audio;
#[path = "subsystem_options/canvas.rs"]
mod canvas;
#[path = "subsystem_options/gpu.rs"]
mod gpu;
#[path = "subsystem_options/support.rs"]
mod support;
#[path = "subsystem_options/webgl_context_extensions.rs"]
mod webgl_context_extensions;
#[path = "subsystem_options/webgl_drawing_queries.rs"]
mod webgl_drawing_queries;
#[path = "subsystem_options/webgl_shaders_buffers.rs"]
mod webgl_shaders_buffers;
#[path = "subsystem_options/webgl_textures_framebuffers.rs"]
mod webgl_textures_framebuffers;

use support::UnusedLoader;

struct HtmlLoader;

#[async_trait::async_trait]
impl ResourceLoader for HtmlLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: HeaderList::new(),
            body: b"<!doctype html><title>storage</title>".to_vec(),
            effective_url: request.url,
        })
    }
}

struct WorkerLoader;

#[async_trait::async_trait]
impl ResourceLoader for WorkerLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let body = if request.url.ends_with("worker.js") {
            b"onmessage = event => postMessage({ doubled: event.data * 2, native: Function.prototype.toString.toString().includes('[native code]') && postMessage.toString().includes('[native code]') })".to_vec()
        } else if request.url.ends_with("shared.js") {
            b"onconnect = event => { const port = event.ports[0]; port.onmessage = message => port.postMessage('shared:' + message.data); }".to_vec()
        } else if request.url.ends_with("shared-state.js") {
            b"let count = 0; onconnect = event => { const port = event.ports[0]; port.onmessage = () => port.postMessage(++count); }".to_vec()
        } else if request.url.ends_with("service.js") {
            b"addEventListener('install', () => postMessage('installed')); addEventListener('fetch', event => event.respondWith(new Response('intercepted')))".to_vec()
        } else if request.url.ends_with("service-state.js") {
            b"let count = 0; addEventListener('fetch', event => event.respondWith(new Response(String(++count))))".to_vec()
        } else if request.url.ends_with("worklet.js") {
            b"if (globalThis.constructor !== PaintWorkletGlobalScope || Object.prototype.toString.call(globalThis) !== '[object PaintWorkletGlobalScope]' || typeof postMessage !== 'undefined' || !WorkerGlobalScope.toString().includes('[native code]')) throw new Error('wrong worklet scope'); registerPaint('brimp-test', class {})".to_vec()
        } else {
            b"<!doctype html><title>worker</title>".to_vec()
        };
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: HeaderList::new(),
            body,
            effective_url: request.url,
        })
    }
}

struct StreamingLoader {
    continue_receiver: Mutex<Option<mpsc::Receiver<()>>>,
}

struct EventSourceLoader {
    continue_receiver: Mutex<Option<mpsc::Receiver<()>>>,
}

struct CancellationLoader {
    cancelled: mpsc::Sender<()>,
}

#[async_trait::async_trait]
impl ResourceLoader for CancellationLoader {
    async fn fetch(&self, _: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        panic!("cancellation loader only supports streaming callbacks")
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::new();
        let callback_handle = handle.clone();
        let cancelled = self.cancelled.clone();
        std::thread::spawn(move || {
            let _ = callback(
                ResourceStreamEvent::Headers {
                    status: StatusCode::OK,
                    headers: HeaderList::new(),
                    url: request.url,
                },
                &callback_handle,
            );
            let _ = callback(
                ResourceStreamEvent::Chunk(b"first".to_vec()),
                &callback_handle,
            );
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            while !callback_handle.is_cancelled() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(1));
            }
            if callback_handle.is_cancelled() {
                let _ = cancelled.send(());
            }
        });
        Ok(handle)
    }
}

#[async_trait::async_trait]
impl ResourceLoader for EventSourceLoader {
    async fn fetch(&self, _: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        panic!("event source loader only supports streaming callbacks")
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let receiver = self.continue_receiver.lock().unwrap().take().unwrap();
        let handle = ResourceStreamHandle::new();
        let callback_handle = handle.clone();
        std::thread::spawn(move || {
            let _ = callback(
                ResourceStreamEvent::Headers {
                    status: StatusCode::OK,
                    headers: HeaderList::new(),
                    url: request.url,
                },
                &callback_handle,
            );
            let _ = callback(
                ResourceStreamEvent::Chunk(b"data: hello\n\n".to_vec()),
                &callback_handle,
            );
            receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            let _ = callback(ResourceStreamEvent::Complete, &callback_handle);
        });
        Ok(handle)
    }
}

#[async_trait::async_trait]
impl ResourceLoader for StreamingLoader {
    async fn fetch(&self, _: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        panic!("streaming loader only supports streaming callbacks")
    }

    fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        mut callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let receiver = self
            .continue_receiver
            .lock()
            .unwrap()
            .take()
            .expect("one streaming request");
        let handle = ResourceStreamHandle::new();
        let callback_handle = handle.clone();
        std::thread::spawn(move || {
            let _ = callback(
                ResourceStreamEvent::Headers {
                    status: StatusCode::OK,
                    headers: HeaderList::new(),
                    url: request.url,
                },
                &callback_handle,
            );
            let _ = callback(
                ResourceStreamEvent::Chunk(b"first".to_vec()),
                &callback_handle,
            );
            receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            let _ = callback(
                ResourceStreamEvent::Chunk(b"second".to_vec()),
                &callback_handle,
            );
            let _ = callback(ResourceStreamEvent::Complete, &callback_handle);
        });
        Ok(handle)
    }
}

#[test]
fn browser_subsystems_are_absent_by_default() {
    let options = PageOptions::default();
    assert!(!options.subsystems().worker_system());
    assert!(!options.subsystems().streaming_networking());
    assert!(options.subsystems().persistent_storage().is_none());
    assert!(!options.subsystems().canvas());
    assert!(!options.subsystems().webgl());
    assert!(!options.subsystems().webgpu());
    assert!(!options.subsystems().webaudio());
    assert!(!options.subsystems().webaudio_output());

    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser.new_page(options).unwrap();
    let observed = page
        .eval(
            r#"JSON.stringify({
                worker: typeof Worker,
                sharedWorker: typeof SharedWorker,
                serviceWorker: "serviceWorker" in navigator,
                webSocket: typeof WebSocket,
                eventSource: typeof EventSource,
                webTransport: typeof WebTransport,
                indexedDB: typeof indexedDB,
                caches: typeof caches,
                storage: "storage" in navigator,
                canvas2d: typeof HTMLCanvasElement.prototype.getContext,
                imageBitmap: typeof createImageBitmap,
                path2d: typeof Path2D,
                webgpu: "gpu" in navigator,
                audioContext: typeof AudioContext,
                mediaStream: typeof MediaStream,
            })"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        observed,
        r#"{"worker":"undefined","sharedWorker":"undefined","serviceWorker":false,"webSocket":"undefined","eventSource":"undefined","webTransport":"undefined","indexedDB":"undefined","caches":"undefined","storage":false,"canvas2d":"undefined","imageBitmap":"undefined","path2d":"undefined","webgpu":false,"audioContext":"undefined","mediaStream":"undefined"}"#,
    );
}

#[test]
fn browser_subsystems_require_explicit_options() {
    let storage = PersistentStorageOptions::new("profile/storage").quota_bytes(4096);
    let options = PageOptions::builder()
        .worker_system(true)
        .streaming_networking(true)
        .persistent_storage(storage.clone())
        .canvas(true)
        .webgl(true)
        .webgpu(true)
        .webaudio(true)
        .build();

    assert!(options.subsystems().worker_system());
    assert!(options.subsystems().streaming_networking());
    assert_eq!(options.subsystems().persistent_storage(), Some(&storage));
    assert!(options.subsystems().canvas());
    assert!(options.subsystems().webgl());
    assert!(options.subsystems().webgpu());
    assert!(options.subsystems().webaudio());
    assert_eq!(storage.root().to_string_lossy(), "profile/storage");
    assert_eq!(storage.quota(), 4096);
}

#[test]
fn graphics_and_audio_options_are_independent() {
    let observed = |options: PageOptions| {
        let features = options.subsystems();
        (
            features.canvas(),
            features.webgl(),
            features.webgpu(),
            features.webaudio(),
            features.webaudio_output(),
        )
    };

    assert_eq!(
        observed(PageOptions::builder().canvas(true).build()),
        (true, false, false, false, false)
    );
    assert_eq!(
        observed(PageOptions::builder().webgl(true).build()),
        (false, true, false, false, false)
    );
    assert_eq!(
        observed(PageOptions::builder().webgpu(true).build()),
        (false, false, true, false, false)
    );
    assert_eq!(
        observed(PageOptions::builder().webaudio(true).build()),
        (false, false, false, true, false)
    );
    assert_eq!(
        observed(PageOptions::builder().webaudio_output(true).build()),
        (false, false, false, true, true)
    );
}

#[test]
fn persistent_storage_apis_store_origin_partitioned_data() {
    let root = std::env::temp_dir().join(format!(
        "brimp-storage-test-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    let options = PageOptions::builder()
        .persistent_storage(PersistentStorageOptions::new(&root).quota_bytes(1024 * 1024))
        .build();
    let browser = Browser::with_resource_loader(Arc::new(HtmlLoader));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let mut first = browser.new_page(options.clone()).unwrap();
    runtime
        .block_on(first.goto("https://storage.test/first"))
        .unwrap();
    assert_eq!(
        first
            .eval(
                "[typeof indexedDB, typeof caches, typeof navigator.storage.getDirectory].join(',')"
            )
            .unwrap()
            .to_string()
            .unwrap(),
        "object,object,function"
    );
    assert_eq!(
        first
            .eval("[indexedDB.open, caches.open, navigator.storage.estimate].every(fn => fn.toString().includes('[native code]'))")
            .unwrap()
            .to_string()
            .unwrap(),
        "true"
    );
    first
        .eval(
            r#"
            globalThis.storageResult = "pending";
            const open = indexedDB.open("app", 1);
            open.onupgradeneeded = () => {
                const store = open.result.createObjectStore("values");
                store.createIndex("byAnswer", "answer", { unique: true });
            };
            open.onsuccess = () => {
                const request = open.result.transaction("values", "readwrite")
                    .objectStore("values").put({ answer: 42 }, "key");
                request.onsuccess = () => storageResult = "written";
            };
            "#,
        )
        .unwrap();
    first.eval("localStorage.pageValue = 'durable'").unwrap();
    assert!(first.run_until_idle_for(Duration::from_secs(1)).unwrap());
    assert_eq!(
        first.eval("storageResult").unwrap().to_string().unwrap(),
        "written"
    );

    let mut second = browser.new_page(options.clone()).unwrap();
    runtime
        .block_on(second.goto("https://storage.test/second"))
        .unwrap();
    assert_eq!(
        second
            .eval("localStorage.pageValue")
            .unwrap()
            .to_string()
            .unwrap(),
        "durable"
    );
    second
        .eval(
            r#"
            globalThis.storageResult = "pending";
            globalThis.indexResult = "pending";
            const open = indexedDB.open("app");
            open.onsuccess = () => {
                const store = open.result.transaction("values").objectStore("values");
                const request = store.get("key");
                request.onsuccess = () => storageResult = String(request.result.answer);
                const indexed = store.index("byAnswer").get(42);
                indexed.onsuccess = () => indexResult = String(indexed.result.answer);
            };
            "#,
        )
        .unwrap();
    assert!(second.run_until_idle_for(Duration::from_secs(1)).unwrap());
    assert_eq!(
        second.eval("storageResult").unwrap().to_string().unwrap(),
        "42"
    );
    assert_eq!(
        second.eval("indexResult").unwrap().to_string().unwrap(),
        "42"
    );
    second
        .eval(
            r#"
            globalThis.storageApis = {};
            caches.open("assets").then(async cache => {
                await cache.put("https://storage.test/item", new Response("cached"));
                storageApis.cache = await (await cache.match("https://storage.test/item")).text();
            });
            navigator.storage.getDirectory().then(async root => {
                const handle = await root.getFileHandle("note.txt", { create: true });
                const writable = await handle.createWritable();
                await writable.write("opfs");
                await writable.close();
                storageApis.opfs = await (await handle.getFile()).text();
            });
            navigator.storage.estimate().then(value => storageApis.quota = value.quota);
            "#,
        )
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while second
        .eval("Object.keys(storageApis).length")
        .unwrap()
        .to_number()
        .unwrap()
        < 3.0
        && std::time::Instant::now() < deadline
    {
        let _ = second.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        second
            .eval("[storageApis.cache, storageApis.opfs, storageApis.quota].join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "cached,opfs,1048576"
    );

    let mut isolated = browser.new_page(options).unwrap();
    runtime
        .block_on(isolated.goto("https://other-origin.test/"))
        .unwrap();
    isolated
        .eval("globalThis.otherDatabases = -1; indexedDB.databases().then(value => otherDatabases = value.length)")
        .unwrap();
    assert_eq!(
        isolated
            .eval("[localStorage.getItem('pageValue'), otherDatabases].join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        ",0"
    );

    let mut limited = browser
        .new_page(
            PageOptions::builder()
                .persistent_storage(
                    PersistentStorageOptions::new(root.join("limited")).quota_bytes(8),
                )
                .build(),
        )
        .unwrap();
    runtime
        .block_on(limited.goto("https://quota.test/"))
        .unwrap();
    assert_eq!(
        limited
            .eval("try { localStorage.setItem('large', '0123456789'); 'missed' } catch (error) { error.name }")
            .unwrap()
            .to_string()
            .unwrap(),
        "QuotaExceededError"
    );

    if root.exists() {
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn enabled_worker_uses_an_isolated_runtime_and_posts_messages() {
    let browser = Browser::with_resource_loader(Arc::new(WorkerLoader));
    let mut page = browser
        .new_page(PageOptions::builder().worker_system(true).build())
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(page.goto("https://worker.test/")).unwrap();
    assert_eq!(
        page.eval(
            "[typeof Worker, typeof SharedWorker, typeof navigator.serviceWorker, typeof Worklet].join(',')"
        )
        .unwrap()
        .to_string()
        .unwrap(),
        "function,function,object,function"
    );
    assert!(
        page.eval("[Worker, Worker.prototype.postMessage, navigator.serviceWorker.register].every(fn => fn.toString().includes('[native code]'))")
            .unwrap()
            .to_string()
            .unwrap()
            == "true"
    );
    page.eval(
        r#"
        globalThis.workerResult = "pending";
        globalThis.workerNative = false;
        const worker = new Worker("/worker.js");
        worker.onmessage = event => { workerResult = String(event.data.doubled); workerNative = event.data.native; };
        worker.onerror = event => workerResult = "error:" + event.message;
        worker.postMessage(21);
        "#,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page.eval("workerResult").unwrap().to_string().unwrap() == "pending"
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("workerResult").unwrap().to_string().unwrap(),
        "42"
    );
    assert_eq!(
        page.eval("String(workerNative)")
            .unwrap()
            .to_string()
            .unwrap(),
        "true"
    );
}

#[test]
fn named_shared_and_service_worker_realms_survive_page_lifetimes() {
    let browser = Browser::with_resource_loader(Arc::new(WorkerLoader));
    let options = PageOptions::builder().worker_system(true).build();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let mut first = browser.new_page(options.clone()).unwrap();
    runtime
        .block_on(first.goto("https://worker.test/first"))
        .unwrap();
    first
        .eval(
            r#"
            globalThis.sharedCount = 0;
            const shared = new SharedWorker("/shared-state.js", { name: "counter" });
            shared.port.onmessage = event => sharedCount = event.data;
            shared.port.postMessage("next");
            globalThis.serviceCount = "pending";
            navigator.serviceWorker.register("/service-state.js", { scope: "/" }).then(() =>
                fetch("/controlled").then(response => response.text()).then(value => serviceCount = value)
            );
            "#,
        )
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while first
        .eval("sharedCount === 0 || serviceCount === 'pending'")
        .unwrap()
        .to_string()
        .unwrap()
        == "true"
        && std::time::Instant::now() < deadline
    {
        let _ = first.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        first
            .eval("[sharedCount, serviceCount].join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "1,1"
    );
    drop(first);

    let mut second = browser.new_page(options).unwrap();
    runtime
        .block_on(second.goto("https://worker.test/second"))
        .unwrap();
    second
        .eval(
            r#"
            globalThis.sharedCount = 0;
            const shared = new SharedWorker("/shared-state.js", { name: "counter" });
            shared.port.onmessage = event => sharedCount = event.data;
            shared.port.postMessage("next");
            globalThis.serviceCount = "pending";
            navigator.serviceWorker.register("/service-state.js", { scope: "/" }).then(() =>
                fetch("/controlled").then(response => response.text()).then(value => serviceCount = value)
            );
            "#,
        )
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while second
        .eval("sharedCount === 0 || serviceCount === 'pending'")
        .unwrap()
        .to_string()
        .unwrap()
        == "true"
        && std::time::Instant::now() < deadline
    {
        let _ = second.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        second
            .eval("[sharedCount, serviceCount].join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "2,2"
    );
}

#[test]
fn shared_service_and_worklet_entry_points_load_and_run() {
    let browser = Browser::with_resource_loader(Arc::new(WorkerLoader));
    let mut page = browser
        .new_page(PageOptions::builder().worker_system(true).build())
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(page.goto("https://worker.test/")).unwrap();
    page.eval(
        r#"
        globalThis.workerKinds = [];
        const shared = new SharedWorker("/shared.js");
        shared.port.onmessage = event => workerKinds.push(event.data);
        shared.port.postMessage("message");
        navigator.serviceWorker.onmessage = event => workerKinds.push(event.data);
        navigator.serviceWorker.register("/service.js").then(async () => {
            workerKinds.push("registered");
            workerKinds.push(await (await fetch("https://worker.test/intercept")).text());
        });
        CSS.paintWorklet.addModule("/worklet.js").then(() => workerKinds.push("worklet"));
        "#,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page
        .eval("workerKinds.length")
        .unwrap()
        .to_number()
        .unwrap()
        < 5.0
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("workerKinds.slice().sort().join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "installed,intercepted,registered,shared:message,worklet"
    );
}

#[test]
fn enabled_fetch_exposes_body_chunks_before_completion() {
    let (continue_sender, continue_receiver) = mpsc::channel();
    let browser = Browser::with_resource_loader(Arc::new(StreamingLoader {
        continue_receiver: Mutex::new(Some(continue_receiver)),
    }));
    let mut page = browser
        .new_page(PageOptions::builder().streaming_networking(true).build())
        .unwrap();
    page.eval(
        r#"
        globalThis.streamResult = [];
        fetch("https://stream.test/data").then(async response => {
            streamResult.push("headers");
            const reader = response.body.getReader();
            const first = await reader.read();
            streamResult.push(new TextDecoder().decode(first.value));
            const second = await reader.read();
            streamResult.push(new TextDecoder().decode(second.value));
            const end = await reader.read();
            streamResult.push(String(end.done));
        });
        "#,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page
        .eval("streamResult.length")
        .unwrap()
        .to_number()
        .unwrap()
        < 2.0
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("streamResult.join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "headers,first"
    );
    continue_sender.send(()).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page
        .eval("streamResult.length")
        .unwrap()
        .to_number()
        .unwrap()
        < 4.0
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("streamResult.join(',')")
            .unwrap()
            .to_string()
            .unwrap(),
        "headers,first,second,true"
    );
}

#[test]
fn cancelling_a_response_body_cancels_its_network_stream() {
    let (cancelled_sender, cancelled_receiver) = mpsc::channel();
    let browser = Browser::with_resource_loader(Arc::new(CancellationLoader {
        cancelled: cancelled_sender,
    }));
    let mut page = browser
        .new_page(PageOptions::builder().streaming_networking(true).build())
        .unwrap();
    page.eval(
        r#"
        globalThis.cancelResult = "pending";
        fetch("https://stream.test/cancel").then(async response => {
            const reader = response.body.getReader();
            await reader.read();
            await reader.cancel("done");
            cancelResult = "cancelled";
        });
        "#,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page.eval("cancelResult").unwrap().to_string().unwrap() == "pending"
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("cancelResult").unwrap().to_string().unwrap(),
        "cancelled"
    );
    cancelled_receiver
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
}

#[test]
fn enabled_page_websocket_uses_curl_impersonate_transport() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        let message = socket.read().unwrap();
        socket.send(message).unwrap();
        socket.close(None).unwrap();
    });
    let browser = Browser::new().unwrap();
    let mut page = browser
        .new_page(PageOptions::builder().streaming_networking(true).build())
        .unwrap();
    page.eval(&format!(
        r#"
        globalThis.webSocketResult = "pending";
        const socket = new WebSocket("ws://{address}/echo");
        socket.onopen = () => socket.send("hello");
        socket.onmessage = event => webSocketResult = event.data;
        "#
    ))
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while page.eval("webSocketResult").unwrap().to_string().unwrap() == "pending"
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("webSocketResult").unwrap().to_string().unwrap(),
        "hello"
    );
    server.join().unwrap();
}

#[test]
fn event_source_dispatches_messages_before_transfer_completion() {
    let (continue_sender, continue_receiver) = mpsc::channel();
    let browser = Browser::with_resource_loader(Arc::new(EventSourceLoader {
        continue_receiver: Mutex::new(Some(continue_receiver)),
    }));
    let mut page = browser
        .new_page(PageOptions::builder().streaming_networking(true).build())
        .unwrap();
    page.eval(
        r#"
        globalThis.eventSourceResult = "pending";
        const source = new EventSource("https://events.test/stream");
        source.onmessage = event => eventSourceResult = event.data;
        "#,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while page.eval("eventSourceResult").unwrap().to_string().unwrap() == "pending"
        && std::time::Instant::now() < deadline
    {
        let _ = page.run_one_task().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        page.eval("eventSourceResult").unwrap().to_string().unwrap(),
        "hello"
    );
    continue_sender.send(()).unwrap();
}

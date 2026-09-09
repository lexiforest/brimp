use async_trait::async_trait;
use brimp_lite_worker::network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use brimp_lite_worker::runtime::{Browser, PageOptions};
use http::{HeaderMap, HeaderValue, StatusCode};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct IdentityLoader {
    requests: Mutex<Vec<ResourceRequest>>,
}
#[async_trait]
impl ResourceLoader for IdentityLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let url = request.url.clone();
        self.requests.lock().unwrap().push(request);
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html"),
        );
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: headers.into(),
            body: b"<!doctype html><title>Persona</title>".to_vec(),
            effective_url: url,
            metadata: brimp_lite_worker::network::ResponseMetadata::default(),
        })
    }
}

#[test]
fn request_and_javascript_observe_one_coherent_identity() {
    let loader = Arc::new(IdentityLoader::default());
    let mut value =
        serde_json::to_value(brimp_lite_worker::persona::PersonaConfig::default()).unwrap();
    for (key, replacement) in serde_json::json!({
        "user_agent": "Mozilla/PersonaBrowser/150",
        "accept": "text/persona",
        "accept_language": "fr-CA,fr;q=0.9",
        "accept_encoding": "gzip, br",
        "sec_ch_ua": "\"Persona Chrome\";v=\"150\"",
        "sec_ch_ua_mobile": "?0",
        "sec_ch_ua_platform": "\"PersonaOS\"",
        "sec_ch_ua_arch": "\"arm\"",
        "sec_ch_ua_bitness": "\"32\"",
        "sec_ch_ua_model": "\"Desktop\"",
        "sec_ch_ua_platform_version": "\"27.1\"",
        "sec_ch_ua_full_version": "\"150.2.3.4\"",
        "sec_ch_ua_full_version_list": "\"Persona Chrome\";v=\"150.2.3.4\""
    })
    .as_object()
    .unwrap()
    {
        value["base_headers"][key] = replacement.clone();
    }
    value["viewport"] = serde_json::json!({"width":900,"height":700,"device_scale_factor":2});
    for (key, replacement) in serde_json::json!({
        "width":1800,"height":1200,"avail_width":1700,"avail_height":1100,
        "avail_left":20,"avail_top":30,"left":5,"top":6,
        "orientation_type":"landscape-secondary","orientation_angle":180
    })
    .as_object()
    .unwrap()
    {
        value["screen"][key] = replacement.clone();
    }
    value["window"] =
        serde_json::json!({"outer_width":920,"outer_height":760,"screen_x":40,"screen_y":50});
    for (key, replacement) in serde_json::json!({
        "platform":"PersonaPlatform","hardware_concurrency":12,"device_memory_gb":16,
        "max_touch_points":3,"connection_rtt_ms":42,"connection_downlink_mbps":"12.5",
        "connection_save_data":true,"do_not_track":"1","vendor":"Persona Vendor",
        "product_sub":"20260827","pdf_viewer_enabled":false
    })
    .as_object()
    .unwrap()
    {
        value["navigator"][key] = replacement.clone();
    }
    value["locale"] = serde_json::json!("fr-CA");
    value["languages"] = serde_json::json!(["fr-CA", "fr"]);
    let persona = brimp_lite_worker::persona::PersonaConfig::from_json(&value.to_string()).unwrap();
    let browser = Browser::with_persona_and_resource_loader(persona, loader.clone()).unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://identity.test/", Duration::from_secs(1))
        .unwrap();
    let observed = page.evaluate("({ userAgent: navigator.userAgent, appVersion: navigator.appVersion, platform: navigator.platform, language: navigator.language, languages: navigator.languages, hardwareConcurrency: navigator.hardwareConcurrency, deviceMemory: navigator.deviceMemory, maxTouchPoints: navigator.maxTouchPoints, connection: { type: navigator.connection.type, rtt: navigator.connection.rtt, downlink: navigator.connection.downlink, effectiveType: navigator.connection.effectiveType, saveData: navigator.connection.saveData }, doNotTrack: navigator.doNotTrack, vendor: navigator.vendor, productSub: navigator.productSub, pdfViewerEnabled: navigator.pdfViewerEnabled, webdriver: navigator.webdriver, uaData: [navigator.userAgentData.platform, navigator.userAgentData.brands[0].brand], viewport: [window.innerWidth, window.innerHeight, window.devicePixelRatio], screen: [screen.width, screen.height, screen.availWidth, screen.availHeight, screen.availLeft, screen.availTop, screen.colorDepth, screen.pixelDepth, screen.orientation.type, screen.orientation.angle], window: [window.outerWidth, window.outerHeight, window.screenX, window.screenY], screenPosition: [screen.left, screen.top], plugins: navigator.plugins.length })").unwrap();
    let request = &loader.requests.lock().unwrap()[0];
    assert_eq!(
        request
            .headers
            .get(http::header::USER_AGENT)
            .unwrap()
            .to_str()
            .unwrap(),
        observed["userAgent"]
    );
    for (name, expected) in [
        ("accept", "text/persona"),
        ("accept-language", "fr-CA,fr;q=0.9"),
        ("accept-encoding", "gzip, br"),
        ("sec-ch-ua", "\"Persona Chrome\";v=\"150\""),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"PersonaOS\""),
        ("sec-ch-ua-arch", "\"arm\""),
    ] {
        assert_eq!(request.headers[name].to_str().unwrap(), expected);
    }
    assert_eq!(observed["appVersion"], "PersonaBrowser/150");
    assert_eq!(observed["platform"], "PersonaPlatform");
    assert_eq!(observed["language"], "fr-CA");
    assert_eq!(observed["languages"], serde_json::json!(["fr-CA", "fr"]));
    assert_eq!(observed["hardwareConcurrency"], 12);
    assert_eq!(observed["deviceMemory"], 16);
    assert_eq!(observed["maxTouchPoints"], 3);
    assert_eq!(
        observed["connection"],
        serde_json::json!({
            "type": "wifi",
            "rtt": 42,
            "downlink": 12.5,
            "effectiveType": "4g",
            "saveData": true
        })
    );
    assert_eq!(observed["doNotTrack"], "1");
    assert_eq!(observed["vendor"], "Persona Vendor");
    assert_eq!(observed["productSub"], "20260827");
    assert_eq!(observed["pdfViewerEnabled"], false);
    assert_eq!(observed["webdriver"], false);
    assert_eq!(
        observed["uaData"],
        serde_json::json!(["PersonaOS", "Persona Chrome"])
    );
    assert_eq!(observed["viewport"], serde_json::json!([900, 700, 2]));
    assert_eq!(
        observed["screen"],
        serde_json::json!([
            1800,
            1200,
            1700,
            1100,
            20,
            30,
            30,
            30,
            "landscape-secondary",
            180
        ])
    );
    assert_eq!(observed["window"], serde_json::json!([920, 760, 40, 50]));
    assert_eq!(observed["plugins"], 0);
    assert_eq!(observed["screenPosition"], serde_json::json!([5, 6]));
    let hints = page.evaluate_remote("navigator.userAgentData.getHighEntropyValues(['architecture','bitness','model','platformVersion','uaFullVersion','fullVersionList'])", true, None, true).unwrap()["value"].clone();
    assert_eq!(
        hints,
        serde_json::json!({
            "brands":[{"brand":"Persona Chrome","version":"150"}], "mobile":false,"platform":"PersonaOS",
            "architecture":"arm","bitness":"32","model":"Desktop","platformVersion":"27.1",
            "uaFullVersion":"150.2.3.4","fullVersionList":[{"brand":"Persona Chrome","version":"150.2.3.4"}]
        })
    );
}

#[test]
fn browser_builtins_have_native_sources_and_web_idl_shapes() {
    let loader = Arc::new(IdentityLoader::default());
    let browser = Browser::with_persona_and_resource_loader(
        brimp_lite_worker::persona::PersonaConfig::default(),
        loader,
    )
    .unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    let observed = page
        .evaluate(
            r#"(() => {
                function authoredFunction() { return 42; }
                const userAgent = Object.getOwnPropertyDescriptor(
                    Navigator.prototype,
                    "userAgent",
                );
                const querySelector = Document.prototype.querySelector;
                const nonNativeBuiltins = [];
                for (const name of Object.getOwnPropertyNames(globalThis)) {
                    const constructor = globalThis[name];
                    if (!/^[A-Z]/.test(name) || typeof constructor !== "function") continue;
                    const constructorSource = Function.prototype.toString.call(constructor);
                    if (!constructorSource.includes("[native code]")) {
                        nonNativeBuiltins.push(name);
                    }
                    const prototype = constructor.prototype;
                    if (!prototype) continue;
                    for (const key of Reflect.ownKeys(prototype)) {
                        const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
                        for (const [kind, fn] of [
                            ["value", descriptor.value],
                            ["get", descriptor.get],
                            ["set", descriptor.set],
                        ]) {
                            if (
                                typeof fn === "function"
                                && !Function.prototype.toString.call(fn).includes("[native code]")
                            ) nonNativeBuiltins.push(`${name}.${String(key)}:${kind}`);
                        }
                    }
                }
                return {
                    navigatorHasOwnUserAgent: Object.hasOwn(navigator, "userAgent"),
                    navigatorGetter: Function.prototype.toString.call(userAgent.get),
                    navigatorDescriptor: {
                        enumerable: userAgent.enumerable,
                        configurable: userAgent.configurable,
                        hasSetter: typeof userAgent.set === "function",
                    },
                    documentConstructor: Function.prototype.toString.call(Document),
                    querySelector: Function.prototype.toString.call(querySelector),
                    functionToString: Function.prototype.toString.call(Function.prototype.toString),
                    authoredFunction: Function.prototype.toString.call(authoredFunction),
                    timeout: Function.prototype.toString.call(setTimeout),
                    uaDataMethod: Function.prototype.toString.call(
                        NavigatorUAData.prototype.getHighEntropyValues,
                    ),
                    screenTag: Object.prototype.toString.call(screen),
                    pluginsTag: Object.prototype.toString.call(navigator.plugins),
                    pluginsPrototype: Object.getPrototypeOf(navigator.plugins) === PluginArray.prototype,
                    pluginsIsArray: Array.isArray(navigator.plugins),
                    pluginMimeRelationship: navigator.plugins.length === 0
                        || navigator.plugins[0][0].enabledPlugin === navigator.plugins[0],
                    uaBrands: navigator.userAgentData.brands,
                    doNotTrack: navigator.doNotTrack,
                    coherentScreen: screen.width >= innerWidth
                        && screen.height >= innerHeight
                        && screen.availWidth <= screen.width
                        && screen.availHeight <= screen.height
                        && screen.colorDepth === screen.pixelDepth,
                    constructorDescriptors: [Document, PluginArray, Screen].map(constructor => {
                        const descriptor = Object.getOwnPropertyDescriptor(
                            globalThis,
                            constructor.name,
                        );
                        return [descriptor.writable, descriptor.enumerable, descriptor.configurable];
                    }),
                    markerLeaked: "__brimpMarkWebBuiltin" in globalThis,
                    hostBridgeLeaked: "__brimp" in globalThis,
                    bindingLexicalsLeaked: typeof __eventListeners !== "undefined",
                    nonNativeBuiltins,
                };
            })()"#,
        )
        .unwrap();

    assert_eq!(observed["navigatorHasOwnUserAgent"], false);
    assert_eq!(
        observed["navigatorGetter"],
        "function get userAgent() { [native code] }"
    );
    assert_eq!(
        observed["navigatorDescriptor"],
        serde_json::json!({
            "enumerable": true,
            "configurable": true,
            "hasSetter": false,
        })
    );
    assert_eq!(
        observed["documentConstructor"],
        "function Document() { [native code] }"
    );
    assert_eq!(
        observed["querySelector"],
        "function querySelector() { [native code] }"
    );
    assert_eq!(
        observed["functionToString"],
        "function toString() { [native code] }"
    );
    assert!(
        observed["authoredFunction"]
            .as_str()
            .unwrap()
            .contains("return 42")
    );
    assert_eq!(
        observed["timeout"],
        "function setTimeout() { [native code] }"
    );
    assert_eq!(
        observed["uaDataMethod"],
        "function getHighEntropyValues() { [native code] }"
    );
    assert_eq!(observed["screenTag"], "[object Screen]");
    assert_eq!(observed["pluginsTag"], "[object PluginArray]");
    assert_eq!(observed["pluginsPrototype"], true);
    assert_eq!(observed["pluginsIsArray"], false);
    assert_eq!(observed["pluginMimeRelationship"], true);
    assert_eq!(
        observed["uaBrands"],
        serde_json::json!([
            {"brand": "Not;A=Brand", "version": "8"},
            {"brand": "Chromium", "version": "150"},
            {"brand": "Google Chrome", "version": "150"},
        ])
    );
    assert_eq!(observed["doNotTrack"], serde_json::Value::Null);
    assert_eq!(observed["coherentScreen"], true);
    assert_eq!(
        observed["constructorDescriptors"],
        serde_json::json!([
            [true, false, true],
            [true, false, true],
            [true, false, true]
        ])
    );
    assert_eq!(observed["markerLeaked"], false);
    assert_eq!(observed["hostBridgeLeaked"], false);
    assert_eq!(observed["bindingLexicalsLeaked"], false);
    assert_eq!(observed["nonNativeBuiltins"], serde_json::json!([]));
}

#[test]
fn navigator_flags_control_supported_apis() {
    let mut persona = brimp_lite_worker::persona::PersonaConfig::default();
    persona.navigator.permissions_enabled = false;
    persona.navigator.bluetooth_enabled = false;
    persona.navigator.media_devices_enabled = false;
    persona.navigator.offscreen_canvas_enabled = false;
    persona.navigator.expose_global_privacy_control = true;
    persona.navigator.global_privacy_control = true;
    let browser =
        Browser::with_persona_and_resource_loader(persona, Arc::new(IdentityLoader::default()))
            .unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    assert_eq!(page.evaluate("({ permissions: 'permissions' in navigator, bluetooth: 'bluetooth' in navigator, mediaDevices: 'mediaDevices' in navigator, offscreen: typeof OffscreenCanvas, transfer: typeof HTMLCanvasElement.prototype.transferControlToOffscreen, privacy: navigator.globalPrivacyControl, canvas: typeof document.createElement('canvas').getContext })").unwrap(), serde_json::json!({
        "permissions":false,"bluetooth":false,"mediaDevices":false,"offscreen":"undefined","transfer":"undefined","privacy":true,"canvas":"function"
    }));
}

#[test]
fn notification_permission_and_bluetooth_availability_follow_persona() {
    let mut persona = brimp_lite_worker::persona::PersonaConfig::default();
    persona.navigator.notification_permission = "denied".into();
    persona.navigator.bluetooth_available = false;
    let browser =
        Browser::with_persona_and_resource_loader(persona, Arc::new(IdentityLoader::default()))
            .unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    assert_eq!(page.evaluate_remote("Promise.all([Notification.requestPermission(), navigator.permissions.query({ name: 'notifications' }).then(p => p.state), navigator.bluetooth.getAvailability(), navigator.mediaDevices.enumerateDevices()])", true, None, true).unwrap()["value"], serde_json::json!(["denied","denied",false,[]]));
}

#[test]
fn example_without_client_hints_omits_them_from_requests_and_navigator() {
    let persona = brimp_lite_worker::persona::PersonaConfig::from_json(include_str!(
        "../../../../examples/cli/safari-persona.json"
    ))
    .unwrap();
    let loader = Arc::new(IdentityLoader::default());
    let browser = Browser::with_persona_and_resource_loader(persona, loader.clone()).unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://identity.test/", Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        page.evaluate("'userAgentData' in navigator").unwrap(),
        false
    );
    let requests = loader.requests.lock().unwrap();
    for (name, _) in requests[0].headers.iter() {
        assert!(!name.as_str().starts_with("sec-ch-ua"));
    }
}

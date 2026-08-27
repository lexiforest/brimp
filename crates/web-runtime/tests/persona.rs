use async_trait::async_trait;
use http::{HeaderMap, HeaderValue, StatusCode};
use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use web_runtime::{AutomationBrowser, PageOptions};

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
        })
    }
}

#[test]
fn request_and_javascript_observe_one_coherent_identity() {
    let loader = Arc::new(IdentityLoader::default());
    let persona = persona::PersonaConfig::from_json(
        r#"{
            "schema_version": 1,
            "transport": { "impersonation_profile": "chrome150" },
            "identity": {
                "brand": "Persona Chrome",
                "version": "150.2.3.4",
                "platform": "PersonaOS",
                "platform_version": "27.1",
                "architecture": "arm",
                "bitness": "64",
                "model": "Desktop"
            },
            "network": {
                "user_agent": "PersonaBrowser/150",
                "accept": "text/persona",
                "accept_language": "fr-CA,fr;q=0.9",
                "accept_encoding": "gzip, br",
                "sec_ch_ua": "\"Persona Chrome\";v=\"150\"",
                "sec_ch_ua_mobile": "?0",
                "sec_ch_ua_platform": "\"PersonaOS\"",
                "sec_ch_ua_arch": "\"arm\""
            },
            "features": {
                "user_agent_data": true,
                "device_memory": true,
                "network_information": true
            },
            "viewport": { "width": 900, "height": 700, "device_scale_factor": 2 },
            "screen": {
                "width": 1800,
                "height": 1200,
                "avail_width": 1700,
                "avail_height": 1100,
                "avail_left": 20,
                "avail_top": 30,
                "color_depth": 30,
                "pixel_depth": 30,
                "orientation_type": "landscape-secondary",
                "orientation_angle": 180
            },
            "window": {
                "outer_width": 920,
                "outer_height": 760,
                "screen_x": 40,
                "screen_y": 50
            },
            "navigator": {
                "app_version": "Persona App/150",
                "platform": "PersonaPlatform",
                "hardware_concurrency": 12,
                "device_memory_gb": 16,
                "max_touch_points": 3,
                "connection_rtt_ms": 42,
                "connection_downlink_mbps": "12.5",
                "connection_effective_type": "4g",
                "connection_save_data": true,
                "do_not_track": "1",
                "vendor": "Persona Vendor",
                "product_sub": "20260827",
                "pdf_viewer_enabled": false
            },
            "chrome": {
                "exposed": true,
                "enumerable": true,
                "runtime": true,
                "app": false,
                "load_times": false,
                "csi": true
            },
            "automation": { "webdriver": true },
            "locale": "fr-CA",
            "languages": ["fr-CA", "fr"]
        }"#,
    )
    .unwrap();
    let browser =
        AutomationBrowser::with_persona_and_resource_loader(persona, loader.clone()).unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    page.navigate("https://identity.test/", Duration::from_secs(1))
        .unwrap();
    let observed = page.evaluate("({ userAgent: navigator.userAgent, appVersion: navigator.appVersion, platform: navigator.platform, language: navigator.language, languages: navigator.languages, hardwareConcurrency: navigator.hardwareConcurrency, deviceMemory: navigator.deviceMemory, maxTouchPoints: navigator.maxTouchPoints, connection: navigator.connection, doNotTrack: navigator.doNotTrack, vendor: navigator.vendor, productSub: navigator.productSub, pdfViewerEnabled: navigator.pdfViewerEnabled, webdriver: navigator.webdriver, uaData: [navigator.userAgentData.platform, navigator.userAgentData.brands[0].brand], viewport: [window.innerWidth, window.innerHeight, window.devicePixelRatio], screen: [screen.width, screen.height, screen.availWidth, screen.availHeight, screen.availLeft, screen.availTop, screen.colorDepth, screen.pixelDepth, screen.orientation.type, screen.orientation.angle], window: [window.outerWidth, window.outerHeight, window.screenX, window.screenY], chrome: [Object.keys(chrome), 'app' in chrome, typeof chrome.csi] })").unwrap();
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
    assert_eq!(observed["appVersion"], "Persona App/150");
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
    assert_eq!(observed["webdriver"], true);
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
    assert_eq!(
        observed["chrome"],
        serde_json::json!([["runtime", "csi"], false, "function"])
    );
}

#[test]
fn structured_browser_apis_return_configured_persona_values() {
    let loader = Arc::new(IdentityLoader::default());
    let persona = persona::PersonaConfig::from_json(
        r#"{
            "schema_version": 1,
            "features": {
                "notifications": true,
                "permissions": true,
                "bluetooth": true,
                "geolocation": true,
                "battery": true,
                "webrtc": true
            },
            "navigator": {
                "notification_permission": "granted",
                "permissions_enabled": true,
                "bluetooth_enabled": true,
                "bluetooth_available": false,
                "media_devices_enabled": true
            },
            "plugins": {
                "entries": [{
                    "name": "Persona PDF",
                    "filename": "persona-pdf",
                    "description": "Persona PDF viewer",
                    "mime_types": [{
                        "type": "application/persona-pdf",
                        "suffixes": "ppdf",
                        "description": "Persona PDF document"
                    }]
                }],
                "pdf_enabled": true,
                "block_system_entries": true
            },
            "media": {
                "speech_voices": ["Fallback Voice"],
                "audio_inputs": 2,
                "video_inputs": 1,
                "audio_outputs": 1,
                "media_source_supported_types": ["video/persona"],
                "decoding_capabilities": [{
                    "content_type": "video/persona",
                    "supported": true,
                    "smooth": true,
                    "power_efficient": false
                }]
            },
            "speech": {
                "voices": [{
                    "voice_uri": "persona-voice",
                    "name": "Persona Voice",
                    "lang": "fr-CA",
                    "default": true,
                    "local_service": false
                }],
                "block_system_voices": true
            },
            "geolocation": {
                "latitude": "25.0330",
                "longitude": "121.5654",
                "accuracy_meters": 7,
                "altitude": "15.5",
                "heading_degrees": "90",
                "speed_meters_per_second": "2.5"
            },
            "battery": {
                "charging": false,
                "charging_time_seconds": 3600,
                "discharging_time_seconds": 7200,
                "level_percent": 42
            },
            "storage": {
                "quota_bytes": 1000000,
                "usage_bytes": 12345,
                "legacy_temporary_quota_bytes": 200000,
                "legacy_persistent_quota_bytes": 800000
            },
            "webrtc": {
                "audio_codecs": ["audio/persona/16000;mode=test"],
                "video_codecs": ["video/persona;profile=test"]
            }
        }"#,
    )
    .unwrap();
    let browser = AutomationBrowser::with_persona_and_resource_loader(persona, loader).unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();

    let synchronous = page
        .evaluate(
            r#"(() => {
                let position;
                navigator.geolocation.getCurrentPosition(value => { position = value; });
                let legacyQuota;
                navigator.webkitTemporaryStorage.queryUsageAndQuota(
                    (usage, quota) => { legacyQuota = [usage, quota]; },
                );
                const voice = speechSynthesis.getVoices()[0];
                const audioCodec = RTCRtpSender.getCapabilities("audio").codecs[0];
                return {
                    plugin: [
                        navigator.plugins.length,
                        navigator.plugins[0].name,
                        navigator.plugins.namedItem("Persona PDF").filename,
                        navigator.mimeTypes.namedItem("application/persona-pdf").suffixes,
                    ],
                    notification: Notification.permission,
                    mediaSource: [
                        MediaSource.isTypeSupported("video/persona"),
                        MediaSource.isTypeSupported("video/unknown"),
                    ],
                    voice: [voice.voiceURI, voice.name, voice.lang, voice.default, voice.localService],
                    position: [
                        position.coords.latitude,
                        position.coords.longitude,
                        position.coords.accuracy,
                        position.coords.altitude,
                        position.coords.heading,
                        position.coords.speed,
                    ],
                    legacyQuota,
                    audioCodec,
                };
            })()"#,
        )
        .unwrap();
    assert_eq!(
        synchronous,
        serde_json::json!({
            "plugin": [1, "Persona PDF", "persona-pdf", "ppdf"],
            "notification": "granted",
            "mediaSource": [true, false],
            "voice": ["persona-voice", "Persona Voice", "fr-CA", true, false],
            "position": [25.033, 121.5654, 7, 15.5, 90, 2.5],
            "legacyQuota": [12345, 200000],
            "audioCodec": {
                "mimeType": "audio/persona",
                "clockRate": 16000,
                "sdpFmtpLine": "mode=test"
            }
        })
    );

    page.evaluate(
        r#"(() => {
            globalThis.__personaAsync = {};
            navigator.permissions.query({ name: "notifications" })
                .then(value => { __personaAsync.permission = value.state; });
            navigator.bluetooth.getAvailability()
                .then(value => { __personaAsync.bluetooth = value; });
            navigator.mediaDevices.enumerateDevices()
                .then(value => { __personaAsync.devices = value.map(device => device.kind); });
            navigator.mediaCapabilities.decodingInfo({ video: { contentType: "video/persona" } })
                .then(value => {
                    __personaAsync.decoding = [value.supported, value.smooth, value.powerEfficient];
                });
            navigator.getBattery()
                .then(value => {
                    __personaAsync.battery = [
                        value.charging,
                        value.chargingTime,
                        value.dischargingTime,
                        value.level,
                    ];
                });
            navigator.storage.estimate()
                .then(value => { __personaAsync.storage = value; });
            return null;
        })()"#,
    )
    .unwrap();
    let asynchronous = page.evaluate("globalThis.__personaAsync").unwrap();
    assert_eq!(
        asynchronous,
        serde_json::json!({
            "permission": "granted",
            "bluetooth": false,
            "devices": ["audioinput", "audioinput", "videoinput", "audiooutput"],
            "decoding": [true, true, false],
            "battery": [false, 3600, 7200, 0.42],
            "storage": { "usage": 12345, "quota": 1000000 }
        })
    );
}

#[test]
fn feature_gates_hide_emulated_apis() {
    let loader = Arc::new(IdentityLoader::default());
    let persona = persona::PersonaConfig::from_json(
        r#"{
            "schema_version": 1,
            "features": {
                "notifications": false,
                "permissions": false,
                "bluetooth": false,
                "geolocation": false,
                "battery": false,
                "webrtc": false
            },
            "navigator": { "media_devices_enabled": false }
        }"#,
    )
    .unwrap();
    let browser = AutomationBrowser::with_persona_and_resource_loader(persona, loader).unwrap();
    let page = browser.new_page(PageOptions::default()).unwrap();
    let observed = page
        .evaluate(
            r#"({
                notification: typeof Notification,
                permissions: "permissions" in navigator,
                bluetooth: "bluetooth" in navigator,
                mediaDevices: "mediaDevices" in navigator,
                geolocation: "geolocation" in navigator,
                battery: "getBattery" in navigator,
                sender: typeof RTCRtpSender,
                receiver: typeof RTCRtpReceiver,
            })"#,
        )
        .unwrap();
    assert_eq!(
        observed,
        serde_json::json!({
            "notification": "undefined",
            "permissions": false,
            "bluetooth": false,
            "mediaDevices": false,
            "geolocation": false,
            "battery": false,
            "sender": "undefined",
            "receiver": "undefined"
        })
    );
}

use crate::*;

#[test]
fn chrome_preset_exposes_expected_identity() {
    let persona = BrowserPersona::from_preset(BrowserPersonaPreset::ChromeStable);

    assert_eq!(persona.browser_name, "Chrome");
    assert_eq!(persona.transport_profile, "chrome150");
    assert_eq!(persona.browser_version, "150.0.0.0");
    assert!(persona.network.user_agent.contains("Chrome/150"));
    assert!(persona.network.sec_ch_ua.contains("v=\"150\""));
    assert_eq!(persona.network.sec_ch_ua_full_version, "\"150.0.0.0\"");
    assert!(
        persona
            .network
            .sec_ch_ua_full_version_list
            .contains("\"Not;A=Brand\";v=\"8.0.0.0\"")
    );
    assert_eq!(persona.network.sec_ch_ua_arch, "\"x86\"");
    assert_eq!(persona.graphics.webgl_vendor, "Intel Inc.");
    assert_eq!(persona.graphics.webgl_masked_vendor, "WebKit");
    assert_eq!(persona.graphics.webgl_masked_renderer, "WebKit WebGL");
    assert_eq!(
        persona.engine.error_messages["number.to_fixed.range"],
        "toFixed() digits argument must be between 0 and 100"
    );
    assert_eq!(
        persona.engine.error_messages["fetch.network"],
        "Failed to fetch"
    );
}

#[test]
fn firefox_preset_uses_firefox_transport_and_hides_chromium_apis() {
    let persona = BrowserPersona::from_preset(BrowserPersonaPreset::FirefoxStable);

    assert_eq!(persona.transport_profile, "firefox_152_macos_26.0");
    assert_eq!(persona.browser_family, "firefox");
    assert!(persona.network.user_agent.contains("Firefox/152.0"));
    assert_eq!(persona.js.app_version, "5.0 (Macintosh)");
    assert_eq!(persona.js.oscpu, "Intel Mac OS X 10.15");
    assert_eq!(persona.js.do_not_track, "unspecified");
    assert!(persona.js.expose_global_privacy_control);
    assert!(!persona.js.global_privacy_control);
    assert_eq!(persona.js.hardware_concurrency, 10);
    assert_eq!(persona.graphics.webgl_vendor, "Apple");
    assert_eq!(persona.graphics.webgl_renderer, "Apple M1, or similar");
    assert_eq!(persona.graphics.webgl_masked_vendor, "Mozilla");
    assert_eq!(
        persona.graphics.webgl_masked_renderer,
        "Apple M1, or similar"
    );
    assert_eq!(persona.viewport.width, 1280);
    assert_eq!(persona.viewport.height, 956);
    assert_eq!(persona.viewport.device_scale_factor, 2);
    assert_eq!(persona.screen.width, 3008);
    assert_eq!(persona.screen.height, 1692);
    assert_eq!(persona.screen.color_depth, 30);
    assert_eq!(persona.window.outer_width, 1280);
    assert_eq!(persona.window.outer_height, 1040);
    assert_eq!(persona.window.screen_x, 4);
    assert_eq!(persona.window.screen_y, 30);
    assert!(persona.network.sec_ch_ua.is_empty());
    assert_eq!(persona.features.user_agent_data, Some(false));
    assert_eq!(persona.features.device_memory, Some(false));
    assert_eq!(persona.features.network_information, Some(false));
    assert_eq!(persona.features.trusted_types, Some(false));
    assert_eq!(persona.features.fetch_later, Some(false));
    assert_eq!(persona.features.servo_internal_apis, Some(false));
    assert_eq!(persona.features.media_tracks, Some(false));
    assert_eq!(persona.features.origin_api, Some(false));
    assert_eq!(persona.features.quota_exceeded_error, Some(false));
    assert_eq!(persona.features.visibility_state_entry, Some(false));
    assert_eq!(persona.features.scoped_custom_element_registry, Some(false));
    assert_eq!(persona.features.page_transition_events, Some(false));
    assert_eq!(persona.features.screen_extended, Some(false));
    assert_eq!(persona.features.moz_window_geometry, Some(true));
    assert_eq!(persona.features.battery, Some(false));
    assert_eq!(persona.features.touch, Some(false));
    assert_eq!(persona.features.service_worker, Some(true));
    assert_eq!(persona.features.gamepad, Some(true));
    assert_eq!(
        persona.webrtc.audio_codecs[0],
        "audio/opus/48000;maxplaybackrate=48000;stereo=1;useinbandfec=1"
    );
    assert_eq!(
        persona.webrtc.video_codecs[0],
        "video/VP8;max-fs=12288;max-fr=60"
    );
    assert_eq!(persona.webrtc.ice_candidate_semantics, "firefox");
    assert_eq!(persona.webrtc.sdp_semantics, "firefox");
    assert!(
        persona
            .media
            .decoding_capabilities
            .iter()
            .all(|capability| capability.power_efficient)
    );
    assert_eq!(persona.chrome.exposed, Some(false));
    assert_eq!(persona.engine.stack_format, "spidermonkey");
}

#[test]
fn partial_feature_overrides_preserve_preset_family_gates() {
    let persona = PersonaConfig {
        preset: BrowserPersonaPreset::FirefoxStable,
        features: Some(FeaturesConfig {
            webgpu: Some(true),
            gamepad: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.features.webgpu, Some(true));
    assert_eq!(persona.features.gamepad, Some(false));
    assert_eq!(persona.features.user_agent_data, Some(false));
    assert_eq!(persona.features.device_memory, Some(false));
    assert_eq!(persona.features.network_information, Some(false));
    assert_eq!(persona.features.trusted_types, Some(false));
}

#[test]
fn persona_config_applies_and_validates_webrtc_semantics() {
    let config = PersonaConfig {
        webrtc: Some(WebRtcConfig {
            ice_candidate_semantics: Some("firefox".to_string()),
            sdp_semantics: Some("firefox".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    config.validate().unwrap();
    let persona = config.resolve();
    assert_eq!(persona.webrtc.ice_candidate_semantics, "firefox");
    assert_eq!(persona.webrtc.sdp_semantics, "firefox");

    let invalid = PersonaConfig {
        webrtc: Some(WebRtcConfig {
            sdp_semantics: Some("servo".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(matches!(
        invalid.validate(),
        Err(PersonaConfigError::InvalidNativeProfile(_))
    ));
}

#[test]
fn stable_seed_is_deterministic() {
    assert_eq!(
        PersonaSeed::from_stable_input("/tmp/profile").as_str(),
        PersonaSeed::from_stable_input("/tmp/profile").as_str()
    );
    assert_ne!(
        PersonaSeed::from_stable_input("/tmp/profile-a").as_str(),
        PersonaSeed::from_stable_input("/tmp/profile-b").as_str()
    );
}

#[test]
fn persona_config_resolves_seed_and_viewport() {
    let persona = PersonaConfig {
        seed: Some(PersonaSeed::new("session-a")),
        viewport: Viewport {
            width: 1280,
            height: 720,
            device_scale_factor: 2,
        },
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.seed.as_str(), "session-a");
    assert_eq!(persona.viewport.width, 1280);
    assert_eq!(persona.screen.avail_height, 720);
    assert_eq!(persona.screen.device_scale_factor, 2);
}

#[test]
fn persona_config_applies_v1_identity_network_and_transport() {
    let persona = PersonaConfig {
        identity: Some(IdentityConfig {
            family: Some("chromium".to_string()),
            brand: Some("Edge".to_string()),
            version: Some("150.0.1.2".to_string()),
            engine: Some("blink".to_string()),
            platform: Some("Windows".to_string()),
            platform_version: Some("10.0.0".to_string()),
            architecture: Some("x86".to_string()),
            bitness: Some("64".to_string()),
            model: Some(String::new()),
        }),
        transport: Some(TransportConfig {
            impersonation_profile: "chrome_150_windows_11".to_string(),
        }),
        network: Some(NetworkConfig {
            user_agent: Some("persona-agent".to_string()),
            accept: Some("text/html".to_string()),
            accept_language: Some("en-GB,en;q=0.9".to_string()),
            sec_ch_ua: Some("\"Chromium\";v=\"150\"".to_string()),
            sec_ch_ua_mobile: Some("?0".to_string()),
            sec_ch_ua_platform: Some("\"Windows\"".to_string()),
            sec_ch_ua_full_version: Some("\"150.0.1.2\"".to_string()),
            sec_ch_ua_full_version_list: Some("\"Chromium\";v=\"150.0.1.2\"".to_string()),
            sec_ch_ua_arch: Some("\"x86\"".to_string()),
            sec_ch_ua_bitness: Some("\"64\"".to_string()),
            sec_ch_ua_platform_version: Some("\"10.0.0\"".to_string()),
            sec_ch_ua_model: Some("\"\"".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.browser_family, "chromium");
    assert_eq!(persona.browser_name, "Edge");
    assert_eq!(persona.browser_version, "150.0.1.2");
    assert_eq!(persona.engine_name, "blink");
    assert_eq!(persona.platform_name, "Windows");
    assert_eq!(persona.js.ua_platform_version, "10.0.0");
    assert_eq!(persona.transport_profile, "chrome_150_windows_11");
    assert_eq!(persona.network.user_agent, "persona-agent");
    assert_eq!(persona.network.accept_header, "text/html");
    assert_eq!(persona.network.accept_language, "en-GB,en;q=0.9");
    assert_eq!(
        persona.network.sec_ch_ua_full_version_list,
        "\"Chromium\";v=\"150.0.1.2\""
    );
    assert_eq!(persona.network.sec_ch_ua_platform_version, "\"10.0.0\"");
}

#[test]
fn persona_config_rejects_unknown_and_unsupported_fields() {
    let root_error =
        serde_json::from_str::<PersonaConfig>(r#"{"schema_version":1,"unknown_surface":true}"#)
            .unwrap_err();
    assert!(root_error.to_string().contains("unknown field"));

    let nested_error = serde_json::from_str::<PersonaConfig>(
        r#"{"schema_version":1,"navigator":{"webgpu_enabled":true}}"#,
    )
    .unwrap_err();
    assert!(nested_error.to_string().contains("webgpu_enabled"));

    let automation_error = serde_json::from_str::<PersonaConfig>(
        r#"{"schema_version":1,"automation":{"webdriver":true}}"#,
    )
    .unwrap_err();
    assert!(automation_error.to_string().contains("automation"));
}

#[test]
fn persona_config_validates_schema_and_geometry() {
    let unsupported = PersonaConfig {
        schema_version: 2,
        ..Default::default()
    };
    assert!(matches!(
        unsupported.validate(),
        Err(PersonaConfigError::UnsupportedSchemaVersion { .. })
    ));

    let invalid_screen = PersonaConfig {
        viewport: Viewport {
            width: 100,
            height: 100,
            device_scale_factor: 1,
        },
        screen: Some(ScreenConfig {
            width: Some(100),
            avail_width: Some(101),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(matches!(
        invalid_screen.validate(),
        Err(PersonaConfigError::InvalidScreen)
    ));
}

#[test]
fn canonical_persona_json_is_valid_v1() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../persona/example.json");
    let config = PersonaConfig::from_json_file(path).unwrap();

    assert_eq!(config.schema_version, PERSONA_SCHEMA_VERSION);
    assert_eq!(
        config
            .transport
            .as_ref()
            .map(|transport| transport.impersonation_profile.as_str()),
        Some("chrome150")
    );
    let persona = config.resolve();
    assert_eq!(persona.network.accept_encoding, "gzip, deflate, br, zstd");
    assert_eq!(persona.plugins.entries.len(), 5);
    assert_eq!(persona.plugins.entries[0].mime_types.len(), 2);
    assert_eq!(persona.graphics.webgl1.parameters["3379"], 16384);
    assert_eq!(persona.speech.voices.len(), 2);
    assert_eq!(persona.geo.latitude.as_deref(), Some("37.7749"));
    assert_eq!(persona.webrtc.ipv4.as_deref(), Some("192.0.2.10"));
    assert_eq!(persona.battery.level_percent, 100);
    assert_eq!(persona.storage.quota_bytes, Some(64_424_509_440));
    assert_eq!(persona.css.media.pointer.as_deref(), Some("fine"));
    assert_eq!(persona.domrect.rounding, "nearest");
    assert_eq!(persona.svg.text_width_factor.as_deref(), Some("0.62"));
    assert_eq!(persona.native_functions.preserve_name, Some(true));
    assert_eq!(persona.noise.enabled, Some(true));
}

#[test]
fn transport_profile_validation_uses_catalog_keys() {
    let directory =
        std::env::temp_dir().join(format!("brimp-persona-catalog-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("fingerprints.json");
    std::fs::write(
        &path,
        r#"{"chrome_150_macos_26.0": {}, "chrome_153_macos_26.0": {}}"#,
    )
    .unwrap();

    PersonaConfig::validate_transport_profile_at("chrome150", &path).unwrap();
    PersonaConfig::validate_transport_profile_at("firefox147", &path).unwrap();
    PersonaConfig::validate_transport_profile_at("chrome_153_macos_26.0", &path).unwrap();
    let error = PersonaConfig::validate_transport_profile_at("missing", &path).unwrap_err();
    assert!(matches!(
        error,
        PersonaConfigError::TransportProfileNotFound(_)
    ));

    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn persona_config_applies_screen_depth_overrides() {
    let persona = PersonaConfig {
        viewport: Viewport {
            width: 1728,
            height: 1117,
            device_scale_factor: 2,
        },
        screen: Some(ScreenConfig {
            avail_height: Some(1079),
            avail_left: Some(0),
            avail_top: Some(25),
            left: Some(0),
            top: Some(0),
            color_depth: Some(30),
            pixel_depth: Some(30),
            is_extended: Some(false),
            orientation_type: Some("landscape-primary".to_string()),
            orientation_angle: Some(0),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.viewport.width, 1728);
    assert_eq!(persona.viewport.device_scale_factor, 2);
    assert_eq!(persona.screen.width, 1728);
    assert_eq!(persona.screen.height, 1117);
    assert_eq!(persona.screen.avail_height, 1079);
    assert_eq!(persona.screen.avail_left, 0);
    assert_eq!(persona.screen.avail_top, 25);
    assert_eq!(persona.screen.left, 0);
    assert_eq!(persona.screen.top, 0);
    assert_eq!(persona.screen.color_depth, 30);
    assert_eq!(persona.screen.pixel_depth, 30);
    assert!(!persona.screen.is_extended);
    assert_eq!(persona.screen.orientation_type, "landscape-primary");
    assert_eq!(persona.screen.orientation_angle, 0);
}

#[test]
fn persona_config_applies_window_overrides() {
    let persona = PersonaConfig {
        viewport: Viewport {
            width: 1710,
            height: 896,
            device_scale_factor: 2,
        },
        window: Some(WindowConfig {
            outer_width: Some(1710),
            outer_height: Some(1017),
            screen_x: Some(0),
            screen_y: Some(23),
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.viewport.width, 1710);
    assert_eq!(persona.viewport.height, 896);
    assert_eq!(persona.window.outer_width, 1710);
    assert_eq!(persona.window.outer_height, 1017);
    assert_eq!(persona.window.screen_x, 0);
    assert_eq!(persona.window.screen_y, 23);
}

#[test]
fn persona_config_applies_css_prefix_overrides() {
    let persona = PersonaConfig {
        css: Some(CssConfig {
            moz_prefix_enabled: Some(true),
            webkit_prefix_enabled: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert!(persona.css.moz_prefix_enabled);
    assert!(!persona.css.webkit_prefix_enabled);
}

#[test]
fn persona_config_applies_graphics_overrides() {
    let persona = PersonaConfig {
        graphics: Some(GraphicsConfig {
            webgl_vendor: Some("Google Inc. (Intel Inc.)".to_string()),
            webgl_renderer: Some(
                "ANGLE (Intel Inc., Intel(R) UHD Graphics 630, OpenGL 4.1)".to_string(),
            ),
            webgl_masked_vendor: Some("WebKit".to_string()),
            webgl_masked_renderer: Some("WebKit WebGL".to_string()),
            webgpu_adapter_vendor: Some(String::new()),
            webgpu_adapter_description: Some("sampled adapter".to_string()),
            webgpu_max_bind_groups_plus_vertex_buffers: Some(24),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.graphics.webgl_vendor, "Google Inc. (Intel Inc.)");
    assert_eq!(
        persona.graphics.webgl_renderer,
        "ANGLE (Intel Inc., Intel(R) UHD Graphics 630, OpenGL 4.1)"
    );
    assert_eq!(persona.graphics.webgl_masked_vendor, "WebKit");
    assert_eq!(persona.graphics.webgl_masked_renderer, "WebKit WebGL");
    assert!(persona.graphics.webgpu_adapter_vendor.is_empty());
    assert_eq!(
        persona.graphics.webgpu_adapter_description,
        "sampled adapter"
    );
    assert_eq!(
        persona.graphics.webgpu_max_bind_groups_plus_vertex_buffers,
        24
    );
}

#[test]
fn persona_config_applies_navigator_overrides() {
    let persona = PersonaConfig {
        navigator: Some(NavigatorConfig {
            oscpu: Some("Windows NT 10.0; Win64; x64".to_string()),
            platform: Some("Win32".to_string()),
            languages: Some(vec!["en-GB".to_string(), "en".to_string()]),
            hardware_concurrency: Some(12),
            device_memory_gb: Some(16),
            max_touch_points: Some(1),
            connection_type: Some("wifi".to_string()),
            connection_rtt_ms: Some(100),
            connection_downlink_mbps: Some("10".to_string()),
            connection_effective_type: Some("4g".to_string()),
            connection_save_data: Some(false),
            notification_permission: Some("denied".to_string()),
            do_not_track: Some("1".to_string()),
            expose_global_privacy_control: Some(true),
            global_privacy_control: Some(true),
            permissions_enabled: Some(false),
            bluetooth_enabled: Some(false),
            bluetooth_available: Some(false),
            media_devices_enabled: Some(false),
            offscreen_canvas_enabled: Some(false),
            ua_platform_version: Some("10.0.0".to_string()),
            ua_architecture: Some("arm".to_string()),
            ua_bitness: Some("64".to_string()),
            ua_model: Some("Mac15,3".to_string()),
            pdf_viewer_enabled: Some(false),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.js.platform, "Win32");
    assert_eq!(persona.js.oscpu, "Windows NT 10.0; Win64; x64");
    assert_eq!(persona.js.language, "en-GB");
    assert_eq!(persona.js.languages, ["en-GB", "en"]);
    assert_eq!(persona.network.accept_language, "en-GB,en;q=0.9");
    assert_eq!(persona.js.hardware_concurrency, 12);
    assert_eq!(persona.js.device_memory_gb, 16);
    assert_eq!(persona.js.max_touch_points, 1);
    assert_eq!(persona.js.connection_type, "wifi");
    assert_eq!(persona.js.connection_rtt_ms, 100);
    assert_eq!(persona.js.connection_downlink_mbps, "10");
    assert_eq!(persona.js.connection_effective_type, "4g");
    assert!(!persona.js.connection_save_data);
    assert_eq!(persona.js.notification_permission, "denied");
    assert_eq!(persona.js.do_not_track, "1");
    assert!(persona.js.expose_global_privacy_control);
    assert!(persona.js.global_privacy_control);
    assert!(!persona.js.permissions_enabled);
    assert!(!persona.js.bluetooth_enabled);
    assert!(!persona.js.bluetooth_available);
    assert!(!persona.js.media_devices_enabled);
    assert!(!persona.js.webgpu_enabled);
    assert!(!persona.js.offscreen_canvas_enabled);
    assert!(!persona.js.service_worker_enabled);
    assert_eq!(persona.js.ua_platform_version, "10.0.0");
    assert_eq!(persona.js.ua_architecture, "arm");
    assert_eq!(persona.js.ua_bitness, "64");
    assert_eq!(persona.js.ua_model, "Mac15,3");
    assert!(!persona.js.pdf_viewer_enabled);
}

#[test]
fn persona_config_applies_media_device_overrides() {
    let persona = PersonaConfig {
        media: Some(MediaConfig {
            speech_voices: Some(vec!["Samantha".to_string(), "Alex".to_string()]),
            audio_inputs: Some(2),
            video_inputs: Some(1),
            audio_outputs: Some(3),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.media.audio_inputs, 2);
    assert_eq!(persona.media.video_inputs, 1);
    assert_eq!(persona.media.audio_outputs, 3);
    assert_eq!(persona.media.speech_voices, ["Samantha", "Alex"]);
}

#[test]
fn persona_config_applies_font_overrides() {
    let persona = PersonaConfig {
        fonts: Some(FontConfig {
            families: Some(vec![
                "Helvetica Neue".to_string(),
                "  ".to_string(),
                "Galvji".to_string(),
            ]),
            seed: Some(PersonaSeed::new("font-test-seed")),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.fonts.families, ["Helvetica Neue", "Galvji"]);
    assert_eq!(persona.fonts.seed.as_str(), "font-test-seed");
}

#[test]
fn persona_config_applies_canvas_overrides() {
    let persona = PersonaConfig {
        canvas: Some(CanvasConfig {
            noise_enabled: Some(false),
            noise_amplitude: Some(2),
            blink_low_entropy_probe: Some(false),
            seed: Some(PersonaSeed::new("canvas-test-seed")),
        }),
        ..Default::default()
    }
    .resolve();

    assert!(!persona.canvas.noise_enabled);
    assert_eq!(persona.canvas.noise_amplitude, 2);
    assert!(!persona.canvas.blink_low_entropy_probe);
    assert_eq!(persona.canvas.seed.as_str(), "canvas-test-seed");
}

#[test]
fn persona_config_applies_domrect_overrides() {
    let persona = PersonaConfig {
        domrect: Some(DomRectConfig {
            enabled: Some(false),
            quantization_steps_per_px: Some(128),
            fill_empty_client_rects: Some(false),
            seed: Some(PersonaSeed::new("domrect-test-seed")),
            ..Default::default()
        }),
        ..Default::default()
    }
    .resolve();

    assert!(!persona.domrect.enabled);
    assert_eq!(persona.domrect.quantization_steps_per_px, 128);
    assert!(!persona.domrect.fill_empty_client_rects);
    assert_eq!(persona.domrect.seed.as_str(), "domrect-test-seed");
}

#[test]
fn persona_config_applies_engine_overrides() {
    let persona = PersonaConfig {
        engine: Some(EngineConfig {
            enabled: Some(false),
            family: Some("javascriptcore".to_string()),
            profile: Some("safari-26".to_string()),
            version: Some("26".to_string()),
            stack_format: Some("javascriptcore".to_string()),
            stack_trace_limit: Some(25),
            async_stack_traces: Some(true),
            chromium_high_resolution_time: Some(false),
            date_now_short_interval_threshold_ms: Some(12),
            date_now_short_interval_scale_percent: Some(40),
            error_messages: [(
                "regexp.invalid_flags".to_string(),
                "Invalid flags".to_string(),
            )]
            .into(),
            builtin_sources: [(
                "Array".to_string(),
                "function Array() { custom source }".to_string(),
            )]
            .into(),
        }),
        ..Default::default()
    }
    .resolve();

    assert!(!persona.engine.enabled);
    assert_eq!(persona.engine.family, "javascriptcore");
    assert_eq!(persona.engine.profile, "safari-26");
    assert_eq!(persona.engine.stack_format, "javascriptcore");
    assert_eq!(persona.engine.stack_trace_limit, 25);
    assert!(persona.engine.async_stack_traces);
    assert!(!persona.engine.chromium_high_resolution_time);
    assert_eq!(persona.engine.date_now_short_interval_threshold_ms, 12);
    assert_eq!(persona.engine.date_now_short_interval_scale_percent, 40);
    assert_eq!(
        persona.engine.error_messages["regexp.invalid_flags"],
        "Invalid flags"
    );
    assert_eq!(
        persona.engine.builtin_sources["Array"],
        "function Array() { custom source }"
    );
}

#[test]
fn resolved_persona_round_trips_through_json() {
    let persona = PersonaConfig {
        seed: Some(PersonaSeed::new("round-trip")),
        ..Default::default()
    }
    .resolve();

    let encoded = serde_json::to_string(&persona).unwrap();
    let decoded: ResolvedPersona = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, persona);
}

#[test]
fn persona_config_applies_locale_timezone_and_accept_language_together() {
    let persona: ResolvedPersona = serde_json::from_str::<PersonaConfig>(
        r#"{
                "schema_version": 1,
                "locale": "fr-FR",
                "languages": ["fr-FR", "fr"],
                "accept_language": "fr-FR,fr;q=0.9",
                "timezone": "Europe/Paris"
            }"#,
    )
    .unwrap()
    .resolve();

    assert_eq!(persona.js.language, "fr-FR");
    assert_eq!(persona.js.languages, ["fr-FR", "fr"]);
    assert_eq!(persona.geo.locale, "fr-FR");
    assert_eq!(persona.network.accept_language, "fr-FR,fr;q=0.9");
    assert_eq!(persona.js.timezone, "Europe/Paris");
    assert_eq!(persona.geo.timezone, "Europe/Paris");
}

#[test]
fn persona_config_derives_languages_from_accept_language() {
    let persona = PersonaConfig {
        accept_language: Some("de-DE,de;q=0.9".to_string()),
        ..Default::default()
    }
    .resolve();

    assert_eq!(persona.js.language, "de-DE");
    assert_eq!(persona.js.languages, ["de-DE", "de"]);
    assert_eq!(persona.geo.locale, "de-DE");
    assert_eq!(persona.network.accept_language, "de-DE,de;q=0.9");
}

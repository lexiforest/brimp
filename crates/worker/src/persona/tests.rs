use super::*;
use serde_json::{Value, json};

fn example() -> Value {
    serde_json::from_str(include_str!("example.json")).unwrap()
}

#[test]
fn bundled_example_is_the_complete_default() {
    let config = PersonaConfig::from_json(include_str!("example.json")).unwrap();
    assert_eq!(config, PersonaConfig::default());
    assert_eq!(serde_json::to_value(config).unwrap(), example());
}

#[test]
fn rejects_fields_outside_the_example_at_every_level() {
    let mut paths = vec![vec![]];
    for (name, value) in example().as_object().unwrap() {
        if value.is_object() {
            paths.push(vec![name.clone()]);
        }
    }
    for path in paths {
        let mut value = example();
        let mut target = &mut value;
        for name in &path {
            target = &mut target[name];
        }
        target["unsupported"] = json!(true);
        assert!(
            PersonaConfig::from_json(&value.to_string()).is_err(),
            "{path:?}"
        );
    }
    for obsolete in [
        "preset",
        "identity",
        "transport",
        "network",
        "features",
        "seed",
        "plugins",
        "chrome",
        "css",
        "media",
        "speech",
        "geolocation",
        "webrtc",
        "battery",
        "storage",
        "fonts",
        "domrect",
        "engine",
        "svg",
        "native_functions",
        "noise",
        "accept_language",
    ] {
        let mut value = example();
        value[obsolete] = json!({});
        assert!(
            PersonaConfig::from_json(&value.to_string()).is_err(),
            "{obsolete}"
        );
    }
    for (section, field) in [
        ("canvas", "seed"),
        ("canvas", "blink_low_entropy_probe"),
        ("screen", "device_scale_factor"),
        ("navigator", "app_version"),
        ("navigator", "languages"),
        ("navigator", "service_worker_enabled"),
    ] {
        let mut value = example();
        value[section][field] = json!(null);
        assert!(
            PersonaConfig::from_json(&value.to_string()).is_err(),
            "{section}.{field}"
        );
    }
}

#[test]
fn requires_complete_configuration() {
    assert!(PersonaConfig::from_json(r#"{"schema_version":1}"#).is_err());
    let mut value = example();
    value["base_headers"]
        .as_object_mut()
        .unwrap()
        .remove("sec_ch_ua_model");
    assert!(PersonaConfig::from_json(&value.to_string()).is_err());
}

#[test]
fn rejects_invalid_values() {
    for (path, replacement) in [
        ("/schema_version", json!(2)),
        ("/network_profile", json!("")),
        ("/viewport/width", json!(0)),
        ("/viewport/device_scale_factor", json!(0)),
        ("/screen/width", json!(0)),
        ("/screen/avail_height", json!(9999)),
        ("/base_headers/accept_encoding", json!("")),
        ("/base_headers/user_agent", json!("a\r\nb")),
        ("/navigator/connection_downlink_mbps", json!("NaN")),
        ("/navigator/connection_downlink_mbps", json!("-1")),
        ("/navigator/connection_effective_type", json!("5g")),
        ("/navigator/notification_permission", json!("prompt")),
        ("/canvas/noise_amplitude", json!(256)),
        ("/languages", json!([])),
        ("/locale", json!("")),
        ("/timezone", json!("")),
    ] {
        let mut value = example();
        *value.pointer_mut(path).unwrap() = replacement;
        assert!(
            PersonaConfig::from_json(&value.to_string()).is_err(),
            "{path}"
        );
    }
}

#[test]
fn base_headers_are_not_rewritten_by_locale_or_languages() {
    let config = PersonaConfig {
        locale: "fr-CA".into(),
        languages: vec!["fr-CA".into()],
        ..Default::default()
    };
    let parsed = PersonaConfig::from_json(&serde_json::to_string(&config).unwrap()).unwrap();
    assert_eq!(parsed.base_headers.accept_language, "en-US,en;q=0.9");
    assert_eq!(parsed, config);
}

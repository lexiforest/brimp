use super::*;
use serde_json::json;

#[test]
fn defaults_are_resolved_from_interface_definitions() {
    let profile = BrowserProfile::default();
    profile.validate().unwrap();
    assert!(profile.values().is_empty());
    assert_eq!(profile.core()["identity"]["platform"], "MacIntel");
    assert_eq!(profile.network_profile(), "safari270");
    assert_eq!(
        serde_json::to_value(profile).unwrap(),
        json!({
            "format": "brimp-profile", "version": 1, "values": {}
        })
    );
}

#[test]
fn partial_profiles_drive_native_headers_and_invalidate_resolved_cache() {
    let mut profile = BrowserProfile::from_json(
        r#"{"values":{"identity":{"user_agent":"Custom/1","languages":["fr-CA","fr"]}}}"#,
    )
    .unwrap();
    let headers = profile.request_headers();
    assert_eq!(headers[0], ("user-agent", "Custom/1".to_owned()));
    assert_eq!(headers[2], ("accept-language", "fr-CA,fr;q=0.9".to_owned()));
    profile.set("identity", "user_agent", json!("Custom/2"));
    profile.validate().unwrap();
    assert_eq!(profile.request_headers()[0].1, "Custom/2");
}

#[test]
fn transport_target_follows_the_user_agent_family_unless_set() {
    let chrome = BrowserProfile::from_json(
        r#"{"values":{"identity":{"user_agent":"Mozilla/5.0 (Macintosh) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36"}}}"#,
    )
    .unwrap();
    assert_eq!(chrome.network_profile(), "chrome150");
    let firefox = BrowserProfile::from_json(
        r#"{"values":{"identity":{"user_agent":"Mozilla/5.0 (Macintosh; rv:146.0) Gecko/20100101 Firefox/146.0"}}}"#,
    )
    .unwrap();
    assert_eq!(firefox.network_profile(), "firefox147");
    let safari = BrowserProfile::default();
    assert_eq!(safari.network_profile(), "safari270");
    // An explicit choice wins, so deliberate mismatch experiments stay possible.
    let explicit = BrowserProfile::from_json(
        r#"{"values":{"identity":{"user_agent":"Mozilla/5.0 (Macintosh) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36"},"network":{"impersonation":"safari270"}}}"#,
    )
    .unwrap();
    assert_eq!(explicit.network_profile(), "safari270");
}

#[test]
fn rejects_invalid_documents_and_interface_values() {
    for value in [
        json!({"schema": 1}),
        json!({"version": 2}),
        json!({"format": "other"}),
        json!({"values": {"identity": null}}),
        json!({"values": {"identity": {"unknown": true}}}),
        json!({"values": {"identity": {"languages": []}}}),
        json!({"values": {"identity": {"user_agent": 1}}}),
        json!({"values": {"identity": {"user_agent": "bad\r\nheader"}}}),
    ] {
        assert!(
            BrowserProfile::from_json(&value.to_string()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn retains_plugin_values_for_validation_after_plugin_installation() {
    let profile = BrowserProfile::from_json(
        r#"{"values":{"custom":{"answer":42}},"capture":{"source":"test"}}"#,
    )
    .unwrap();
    assert_eq!(profile.values()["custom"]["answer"], 42);
    assert_eq!(profile.capture().unwrap()["source"], "test");
}

//! Page profile installation and identity configuration.

use super::Page;
use brimp_jsc::{JsException, JsRuntime, ProtectedJsObject};

impl Page {
    pub(super) fn profile_command(&self, command: &str) -> Result<serde_json::Value, JsException> {
        let json = self
            .js
            .call_function_with_string(&self.profile_controller, command)?
            .to_string()?;
        serde_json::from_str(&json).map_err(|e| JsException::from_message(e.to_string()))
    }

    /// The contract registered by core and the installed plugins. No page global is exposed.
    pub fn profile_contract(&self) -> Result<serde_json::Value, JsException> {
        self.profile_command("contract")
    }

    /// Unspecified values use the defaults listed here; capture omissions remain explicit.
    pub fn profile_report(&self) -> Result<serde_json::Value, JsException> {
        self.profile_command("report")
    }

    pub(crate) fn set_identity(
        &mut self,
        identity: crate::runtime::native::IdentityOptions,
    ) -> Result<(), crate::runtime::AutomationError> {
        let mut profile = self.profile.clone();
        identity.apply(&mut profile)?;
        let mut headers = Vec::new();
        let mut values = serde_json::Map::new();
        if identity.user_agent.is_some() {
            headers.push((
                "user-agent".into(),
                profile.core()["identity"]["user_agent"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            ));
            values.insert(
                "userAgent".into(),
                serde_json::json!(profile.core()["identity"]["user_agent"]),
            );
            values.insert(
                "appVersion".into(),
                serde_json::json!(profile.core()["identity"]["app_version"]),
            );
        }
        if identity.languages.is_some() {
            let accept_language = profile
                .request_headers()
                .into_iter()
                .find(|(name, _)| *name == "accept-language")
                .map(|(_, value)| value)
                .unwrap_or_default();
            headers.push(("accept-language".into(), accept_language));
            values.insert(
                "language".into(),
                serde_json::json!(profile.core()["identity"]["locale"]),
            );
            values.insert(
                "languages".into(),
                serde_json::json!(profile.core()["identity"]["languages"]),
            );
        }
        if identity.platform.is_some() {
            values.insert(
                "platform".into(),
                serde_json::json!(profile.core()["identity"]["platform"]),
            );
        }
        self.browsing_context
            .update_identity_headers(headers)
            .map_err(crate::runtime::AutomationError::InvalidInput)?;
        self.profile = profile;
        self.set_navigator_identity_override(serde_json::Value::Object(values).to_string())
            .map_err(|error| crate::runtime::AutomationError::JavaScript(error.to_string()))
    }

    pub(crate) fn set_navigator_identity_override(
        &mut self,
        identity_override: String,
    ) -> Result<(), JsException> {
        self.js
            .call_function_with_string(&self.profile_identity_setter, &identity_override)?;
        let values =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&identity_override)
                .map_err(|error| JsException::from_message(error.to_string()))?;
        for (name, value) in values {
            if value.is_null() {
                self.navigator_identity_override.remove(&name);
            } else {
                self.navigator_identity_override.insert(name, value);
            }
        }
        Ok(())
    }
}

pub(super) fn install_profile(
    runtime: &JsRuntime,
    profile: &crate::profile::BrowserProfile,
) -> Result<(ProtectedJsObject, ProtectedJsObject), JsException> {
    let profile = serde_json::to_string(profile)
        .map_err(|error| JsException::from_message(error.to_string()))?;
    let registry = crate::profile::REGISTRY_SOURCE;
    let definitions = crate::profile::CORE_SOURCE;
    let installer = include_str!("../../../js/internal/profile.js");
    let setter = runtime.eval(&format!(
        "(() => {{ const registry=({registry})({profile}); Object.defineProperty(globalThis,'__brimpProfileRegistry',{{value:registry,configurable:true}}); return ({installer})(({definitions})(registry.forOwner('core'))); }})()"
    ))?.to_object()?;
    let controller = runtime.eval("((registry) => (command) => { if(command==='finish'){registry.validate();delete globalThis.__brimpProfileRegistry;return 'null';} return JSON.stringify(command==='contract'?registry.contract():command==='resolved'?registry.resolved():registry.report()); })(globalThis.__brimpProfileRegistry)")?.to_object()?;
    Ok((setter, controller))
}

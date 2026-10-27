//! Host references to JavaScript values and DOM elements; roots stay on the owner thread.

use super::{FileUpload, HandleToken, NativePage, Operation};
use crate::runtime::AutomationError;
use crate::runtime::bridge::{BridgeEndpoint, BridgeValue, Reference};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Handle {
    endpoint: BridgeEndpoint,
    token: HandleToken,
    disposed: AtomicBool,
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.disposed.swap(true, Ordering::AcqRel) {
            self.endpoint.release(&self.token);
        }
    }
}

/// A rooted value, including primitives. Clones share explicit disposal.
#[derive(Clone)]
pub struct JSHandle(Arc<Handle>, pub(super) Option<NativePage>);

impl JSHandle {
    fn token(&self) -> Result<HandleToken, AutomationError> {
        if self.0.disposed.load(Ordering::Acquire) {
            return Err(AutomationError::StaleHandle);
        }
        self.0.endpoint.check()?;
        Ok(self.0.token)
    }

    pub fn from_reference(endpoint: BridgeEndpoint, token: Reference) -> Self {
        Self(
            Arc::new(Handle {
                endpoint,
                token,
                disposed: AtomicBool::new(false),
            }),
            None,
        )
    }
    pub fn reference(&self) -> Result<Reference, AutomationError> {
        self.token()
    }
    pub fn endpoint(&self) -> BridgeEndpoint {
        self.0.endpoint.clone()
    }
    pub fn peer_handle(&self, token: Reference) -> Self {
        let mut handle = Self::from_reference(self.endpoint(), token);
        handle.1 = self.1.clone();
        handle
    }

    pub fn bridge_command(&self, mut command: Value) -> Result<BridgeValue, AutomationError> {
        command["token"] = json!(self.token()?);
        self.0.endpoint.request(command)
    }
    pub fn value(&self) -> Result<BridgeValue, AutomationError> {
        self.bridge_command(json!({"op":"value"}))
    }
    pub fn json_value(&self) -> Result<Value, AutomationError> {
        match self.bridge_command(json!({"op":"json"}))? {
            BridgeValue::String(value) => {
                serde_json::from_str(&value).map_err(|e| AutomationError::Internal(e.to_string()))
            }
            _ => unreachable!(),
        }
    }
    pub fn call(&self, function: &str, arguments: Vec<Value>) -> Result<JSHandle, AutomationError> {
        self.call_bridge(
            function,
            arguments.into_iter().map(BridgeValue::from_json).collect(),
        )
    }
    pub fn call_bridge(
        &self,
        function: &str,
        arguments: Vec<BridgeValue>,
    ) -> Result<JSHandle, AutomationError> {
        self.command_handle(json!({"op":"call","source":function,"args":arguments,"handle":true}))
    }
    pub fn command_handle(&self, command: Value) -> Result<JSHandle, AutomationError> {
        match self.bridge_command(command)? {
            BridgeValue::Reference(token) => Ok(Self(
                Arc::new(Handle {
                    endpoint: self.endpoint(),
                    token,
                    disposed: AtomicBool::new(false),
                }),
                self.1.clone(),
            )),
            _ => Err(AutomationError::Internal("expected a reference".into())),
        }
    }
    pub fn property(&self, name: &str) -> Result<JSHandle, AutomationError> {
        self.command_handle(json!({"op":"property","name":name,"handle":true}))
    }
    pub fn property_names(&self) -> Result<Vec<String>, AutomationError> {
        match self.bridge_command(json!({"op":"names"}))? {
            BridgeValue::Array(values) => values
                .into_iter()
                .map(|value| match value {
                    BridgeValue::String(v) => Ok(v),
                    _ => Err(AutomationError::Internal("expected property name".into())),
                })
                .collect(),
            _ => unreachable!(),
        }
    }
    pub fn dispose(&self) -> Result<(), AutomationError> {
        if !self.0.disposed.swap(true, Ordering::AcqRel) {
            self.0.endpoint.release(&self.0.token);
        }
        Ok(())
    }
    fn page(&self) -> Result<&NativePage, AutomationError> {
        self.1.as_ref().ok_or_else(|| {
            AutomationError::Unsupported("use reference calls for plugin DOM handles".into())
        })
    }
}

#[derive(Clone)]
pub struct ElementHandle(pub JSHandle);

impl ElementHandle {
    fn input(&self, command: Value) -> Result<(), AutomationError> {
        self.0.page()?.operation(Operation::Input {
            target: Some(self.0.token()?),
            command,
        })?;
        Ok(())
    }
    pub fn focus(&self) -> Result<(), AutomationError> {
        self.input(json!({"action":"focus"}))
    }
    pub fn click(&self) -> Result<(), AutomationError> {
        self.input(json!({"action":"click"}))
    }
    pub fn set_input_files(&self, files: Vec<FileUpload>) -> Result<(), AutomationError> {
        self.input(json!({"action":"files","files":files}))
    }

    pub fn get_attribute(&self, name: &str) -> Result<Option<String>, AutomationError> {
        let value = self
            .0
            .call("(node, name) => node.getAttribute(name)", vec![json!(name)])?
            .json_value()?;
        serde_json::from_value(value).map_err(|error| AutomationError::Internal(error.to_string()))
    }
    pub fn set_attribute(&self, name: &str, value: &str) -> Result<(), AutomationError> {
        self.0.call(
            "(node, name, value) => { node.setAttribute(name, value); return null; }",
            vec![json!(name), json!(value)],
        )?;
        Ok(())
    }
    pub fn text_content(&self) -> Result<String, AutomationError> {
        self.string_property("textContent")
    }
    pub fn html(&self) -> Result<String, AutomationError> {
        self.string_property("outerHTML")
    }
    fn string_property(&self, name: &str) -> Result<String, AutomationError> {
        let value = self.0.property(name)?.json_value()?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| AutomationError::Internal(format!("{name} is not a string")))
    }
    pub fn set_text_content(&self, text: &str) -> Result<(), AutomationError> {
        self.0.call(
            "(node, text) => { node.textContent = text; return null; }",
            vec![json!(text)],
        )?;
        Ok(())
    }
    pub fn remove(&self) -> Result<(), AutomationError> {
        self.0
            .call("node => { node.remove(); return null; }", vec![])?;
        Ok(())
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>, AutomationError> {
        Ok(self
            .0
            .page()?
            .query(Some(self.0.token()?), selector, false)?
            .into_iter()
            .next())
    }
    pub fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<ElementHandle>, AutomationError> {
        self.0.page()?.query(Some(self.0.token()?), selector, true)
    }
}

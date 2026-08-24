use std::{cell::RefCell, collections::HashMap};

use browser_dom::NodeId;
use jsc::{JsException, JsObject, JsObjectIdentity, JsRuntime, NativeCall, ProtectedJsObject};

#[derive(Default)]
pub struct WrapperCache {
    entries: RefCell<WrapperEntries>,
}

#[derive(Default)]
struct WrapperEntries {
    by_node: HashMap<NodeId, ProtectedJsObject>,
    by_object: HashMap<JsObjectIdentity, NodeId>,
}

impl WrapperCache {
    pub fn wrap<'runtime>(
        &self,
        runtime: &'runtime JsRuntime,
        node_id: NodeId,
    ) -> Result<JsObject<'runtime>, JsException> {
        if !self.entries.borrow().by_node.contains_key(&node_id) {
            let wrapper = runtime.make_object()?;
            self.insert(node_id, wrapper);
        }

        Ok(self
            .entries
            .borrow()
            .by_node
            .get(&node_id)
            .expect("wrapper was inserted")
            .handle(runtime))
    }

    pub fn wrap_with_prototype(
        &self,
        call: &NativeCall<'_>,
        node_id: NodeId,
        prototype: JsObjectIdentity,
    ) -> JsObjectIdentity {
        if !self.entries.borrow().by_node.contains_key(&node_id) {
            self.insert(node_id, call.make_object_with_prototype(prototype));
        }
        self.entries.borrow().by_node[&node_id].identity()
    }

    pub fn wrap_with_runtime_prototype<'runtime>(
        &self,
        runtime: &'runtime JsRuntime,
        node_id: NodeId,
        prototype: JsObjectIdentity,
    ) -> JsObject<'runtime> {
        if !self.entries.borrow().by_node.contains_key(&node_id) {
            self.insert(node_id, runtime.make_object_with_prototype(prototype));
        }
        self.entries.borrow().by_node[&node_id].handle(runtime)
    }

    pub fn node_id(&self, object: JsObjectIdentity) -> Option<NodeId> {
        self.entries.borrow().by_object.get(&object).copied()
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().by_node.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.borrow().by_node.is_empty()
    }

    pub fn clear(&self) {
        let mut entries = self.entries.borrow_mut();
        entries.by_object.clear();
        entries.by_node.clear();
    }

    pub fn remove_nodes(&self, node_ids: &[NodeId]) {
        let mut entries = self.entries.borrow_mut();
        for node_id in node_ids {
            if let Some(wrapper) = entries.by_node.remove(node_id) {
                entries.by_object.remove(&wrapper.identity());
            }
        }
    }

    fn insert(&self, node_id: NodeId, wrapper: ProtectedJsObject) {
        let identity = wrapper.identity();
        let mut entries = self.entries.borrow_mut();
        entries.by_object.insert(identity, node_id);
        entries.by_node.insert(node_id, wrapper);
    }
}

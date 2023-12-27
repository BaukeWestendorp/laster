use crate::arena::{NodeArena, NodeId};
use crate::parser::Namespace;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Element {
        namespace_uri: Option<String>,
        prefix: Option<String>,
        local_name: String,
        tag_name: String,
    },
    Document,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    document: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) parent: Option<NodeId>,
}

impl Node {
    /// https://dom.spec.whatwg.org/#concept-create-element
    pub fn create_element(
        document: NodeId,
        local_name: String,
        namespace: Namespace,
        prefix: Option<String>,
        _is: Option<String>,
        _synchronous_custom_elements: bool,
    ) -> Self {
        // TODO: This is not spec compliant at all.

        Self {
            kind: NodeKind::Element {
                namespace_uri: Some(namespace.url().to_string()),
                prefix,
                local_name: local_name.clone(),
                tag_name: local_name,
            },
            document: Some(document),
            children: vec![],
            parent: Some(document),
        }
    }

    pub fn create_document() -> Self {
        // TODO: This is not spec compliant
        Self {
            kind: NodeKind::Document,
            document: None,
            children: vec![],
            parent: None,
        }
    }

    pub fn node_document(&self, arena: &NodeArena) -> NodeId {
        match self.document {
            Some(document) => document,
            None => arena.get_node_id(self),
        }
    }

    pub fn is_element_in_namespace(&self, namespace: Namespace) -> bool {
        if let NodeKind::Element { namespace_uri, .. } = &self.kind {
            return *namespace_uri == Some(namespace.url().to_string());
        }
        false
    }
}

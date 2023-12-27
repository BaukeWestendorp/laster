use crate::arena::{NodeArena, NodeId};
use crate::parser::Namespace;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Document,
    Element {
        namespace_uri: Option<String>,
        prefix: Option<String>,
        local_name: String,
        tag_name: String,
    },
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

    pub fn is_element_with_tag_name(&self, tag_name: &str) -> bool {
        if let NodeKind::Element { tag_name: name, .. } = &self.kind {
            return name == tag_name;
        }
        false
    }

    pub fn is_element_with_one_of_tag_names(&self, tag_names: &[&str]) -> bool {
        if let NodeKind::Element { tag_name: name, .. } = &self.kind {
            return tag_names.contains(&name.as_str());
        }
        false
    }

    pub fn dump(&self, arena: &NodeArena) {
        self.internal_dump(arena, 0);
    }

    fn internal_dump(&self, arena: &NodeArena, indent: usize) {
        let indent_string = " ".repeat(indent * 2);

        println!("{indent_string}{}", self);
        for child in &self.children {
            let child = arena.get_node(*child);
            child.internal_dump(arena, indent + 1);
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            NodeKind::Document => write!(f, "Document"),
            NodeKind::Element { tag_name, .. } => write!(f, "<{}>", tag_name),
        }
    }
}

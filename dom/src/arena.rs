use crate::node::Node;

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub struct NodeArena {
    nodes: Vec<Node>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn create_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn get_node(&self, node_id: NodeId) -> &Node {
        self.nodes.get(node_id).unwrap()
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node {
        self.nodes.get_mut(node_id).unwrap()
    }

    pub fn get_node_id(&self, node: &Node) -> usize {
        self.nodes.iter().position(|n| n == node).unwrap()
    }
}
/// # Mutation Algorithms
///
/// https://dom.spec.whatwg.org/#mutation-algorithms
impl NodeArena {
    /// https://dom.spec.whatwg.org/#concept-node-pre-insert
    pub fn pre_insert(
        &mut self,
        node: NodeId,
        into_parent: NodeId,
        before_child: Option<NodeId>,
    ) -> NodeId {
        // TODO: Ensure pre-insertion validity of node into parent before child.

        // Let referenceChild be child.
        let reference_child = before_child;

        // TODO: If referenceChild is node, then set referenceChild to node’s
        // next sibling.

        // Insert node into parent before referenceChild.
        self.insert(node, into_parent, reference_child);

        // Return node.
        node
    }

    /// https://dom.spec.whatwg.org/#concept-node-insert
    pub fn insert(&mut self, node: NodeId, into_parent: NodeId, before_child: Option<NodeId>) {
        // TODO: This is not spec compliant.

        let parent_node = self.get_node_mut(into_parent);

        if let Some(before_child) = before_child {
            let before_child_index =
                match parent_node.children.iter().position(|n| *n == before_child) {
                    Some(before_child_index) => before_child_index,
                    None => parent_node.children.len() - 1,
                };

            parent_node.children.insert(before_child_index, node);
        } else {
            parent_node.children.push(node);
        }

        let node = self.get_node_mut(node);
        node.parent = Some(into_parent);
    }

    /// https://dom.spec.whatwg.org/#concept-node-append
    pub fn append(&mut self, node: NodeId, into_parent: NodeId) -> NodeId {
        // To append a node to a parent, pre-insert node into parent before null.
        self.pre_insert(node, into_parent, None)
    }
}

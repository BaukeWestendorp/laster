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

    pub fn get_node_id(&self, node: &Node) -> NodeId {
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

    pub fn previous_sibling(&self, node: NodeId) -> Option<NodeId> {
        // FIXME: store previous sibling in node
        if let Some(parent) = self.nodes[node].parent() {
            let children = self.nodes[parent].children();
            let index = children.iter().position(|child| *child == node);
            if let Some(index) = index {
                if index > 0 {
                    return Some(children[index - 1]);
                }
            }
        }
        None
    }

    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        // FIXME: store previous sibling in node
        if let Some(parent) = self.nodes[node].parent() {
            let children = self.nodes[parent].children();
            let index = children.iter().position(|child| *child == node);
            if let Some(index) = index {
                if index < children.len() - 1 {
                    return Some(children[index + 1]);
                }
            }
        }
        None
    }

    /// https://dom.spec.whatwg.org/#concept-node-insert
    pub fn insert(&mut self, node: NodeId, into_parent: NodeId, before_child: Option<NodeId>) {
        // TODO: Let nodes be node’s children, if node is a DocumentFragment node;
        // otherwise « node ».
        let nodes = vec![node];

        // Let count be nodes’s size.
        let count = nodes.len();

        // If count is 0, then return.
        if count == 0 {
            return;
        }

        // TODO:  If node is a DocumentFragment node, then:

        // TODO: If child is non-null, then:

        // TODO: Let previousSibling be child’s previous sibling or parent’s last child
        // if child is null.

        // For each node in nodes, in tree order:
        for node in nodes.iter() {
            // Adopt node into parent’s node document.
            self.adopt(*node, self.get_node(into_parent).node_document(self));

            if let Some(before_child) = before_child {
                // Otherwise, insert node into parent’s children before child’s
                // index.
                let index = self
                    .get_node_mut(into_parent)
                    .children
                    .iter()
                    .position(|n| *n == before_child)
                    .unwrap();
                self.get_node_mut(into_parent).children.insert(index, *node);
            } else {
                // If child is null, then append node to parent’s children.
                self.get_node_mut(into_parent).children.push(*node);
            }

            // TODO: If parent is a shadow host whose shadow root’s slot
            // assignment is "named" and node is a slottable, then
            // assign a slot for node.

            // TODO: If parent’s root is a shadow root, and parent is a slot
            // whose assigned nodes is the empty list, then run
            // signal a slot change for parent.

            // TODO: Run assign slottables for a tree with node’s root.

            // TODO: For each shadow-including inclusive descendant
            // inclusiveDescendant of node, in shadow-including tree order:
        }

        // TODO: If suppress observers flag is unset, then queue a tree mutation
        // record for parent with nodes, « », previousSibling, and child.

        // TODO: Run the children changed steps for parent.
    }

    /// https://dom.spec.whatwg.org/#concept-node-append
    pub fn append(&mut self, node: NodeId, into_parent: NodeId) -> NodeId {
        // To append a node to a parent, pre-insert node into parent before null.
        self.pre_insert(node, into_parent, None)
    }

    /// https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(&mut self, node: NodeId, document: NodeId) {
        // Let oldDocument be node’s node document.
        let old_document = self.get_node(node).node_document(self);

        // TODO: If node’s parent is non-null, then remove node.
        if self.get_node(node).parent().is_some() {
            todo!();
        }

        // If document is not oldDocument, then:
        if document != old_document {
            // TODO: This is not spec compliant.
            let children = self.get_node(node).children().to_vec();
            for child in children.iter() {
                self.get_node_mut(*child).document = Some(document);
            }
        }
    }
}

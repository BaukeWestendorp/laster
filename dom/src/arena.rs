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
}

use imnodes::{AttributeId, InputPinId, NodeId, OutputPinId, Link};
use lifec::{Component, DenseVecStorage, Sequence};

/// Struct for the node state of a sequence,
///
#[derive(Component, Clone, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct NodeContext(
    pub Sequence,
    pub Option<NodeId>,
    pub Option<InputPinId>,
    pub Option<OutputPinId>,
    pub Option<AttributeId>,
);

impl NodeContext {
    /// Return the node id for this context,
    /// 
    pub fn node_id(&self) -> Option<NodeId> {
        self.1
    }

    /// Create a link struct between two contexts,
    /// 
    pub fn link(&self, other: &NodeContext) -> Option<Link> {
        if let (
            NodeContext(_, Some(start_node), Some(_), Some(start_pin), ..),
            NodeContext(_, Some(end_node), Some(end_pin), Some(_), ..),
        ) = (self, other)
        {
            let start_node = *start_node;
            let end_node = *end_node;
            let start_pin = *start_pin;
            let end_pin = *end_pin;

            Some(Link {
                start_node,
                end_node,
                start_pin,
                end_pin,
                craeated_from_snap: true,
            })
        } else {
            None
        }
    }
}

use imnodes::{AttributeId, InputPinId, Link, NodeId, OutputPinId};
use lifec::prelude::{Component, DenseVecStorage, Sequence};

/// Struct for the node state of a sequence,
///
#[derive(Component, Clone, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct NodeContext {
    pub node_id: Option<NodeId>,
    pub sequence: (
        Sequence,
        Option<InputPinId>,
        Option<OutputPinId>,
        Option<AttributeId>,
    ),
}

impl NodeContext {
    /// Return the node id for this context,
    ///
    pub fn node_id(&self) -> Option<NodeId> {
        self.node_id
    }

    /// Create a link struct between two contexts,
    ///
    pub fn link(&self, other: &NodeContext) -> Option<Link> {
        if let (
            NodeContext {
                node_id: Some(start_node),
                sequence: (_, Some(_), Some(start_pin), ..),
            },
            NodeContext {
                node_id: Some(end_node),
                sequence: (_, Some(end_pin), Some(_), ..),
                ..
            },
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

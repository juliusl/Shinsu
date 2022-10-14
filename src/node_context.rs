use imnodes::{NodeId, InputPinId, OutputPinId, AttributeId};
use lifec::{Sequence, Component, DenseVecStorage};

/// Struct for all node id's and sequence this node represents,
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
    pub fn node_id(&self) -> Option<NodeId> {
        self.1
    }
}

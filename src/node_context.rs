use imnodes::{AttributeId, IdentifierGenerator, InputPinId, Link, NodeId, OutputPinId};
use lifec::prelude::{Component, DenseVecStorage};
use specs::Entity;

/// Struct for the node state of a sequence,
///
#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct NodeContext {
    pub entity: Entity,
    pub node_id: NodeId,
    pub input_pin_id: InputPinId,
    pub output_pin_id: OutputPinId,
    pub attribute_id: AttributeId,
    pub row: usize,
    pub col: usize,
}

impl NodeContext {
    /// Returns a new node context,
    ///
    pub fn new(entity: Entity, idgen: &mut IdentifierGenerator) -> Self {
        NodeContext {
            entity,
            node_id: idgen.next_node(),
            input_pin_id: idgen.next_input_pin(),
            output_pin_id: idgen.next_output_pin(),
            attribute_id: idgen.next_attribute(),
            row: 0,
            col: 0,
        }
    }

    /// Return the node id for this context,
    ///
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Returns the input pin id for this context,
    ///
    pub fn input_pin_id(&self) -> InputPinId {
        self.input_pin_id
    }

    /// Returns the output pin id for this context,
    ///
    pub fn output_pin_id(&self) -> OutputPinId {
        self.output_pin_id
    }

    /// Returns the attribute id for this context,
    ///
    pub fn attribute_id(&self) -> AttributeId {
        self.attribute_id
    }

    /// Returns a link
    ///
    pub fn link(&self, other: &NodeContext, craeated_from_snap: bool) -> Link {
        let NodeContext {
            node_id: from_node_id,
            output_pin_id: from_output_pin_id,
            ..
        } = self;

        let NodeContext {
            node_id: to_node_id,
            input_pin_id: to_input_pin_id,
            ..
        } = other;

        Link {
            start_node: *from_node_id,
            end_node: *to_node_id,
            start_pin: *from_output_pin_id,
            end_pin: *to_input_pin_id,
            craeated_from_snap,
        }
    }
}

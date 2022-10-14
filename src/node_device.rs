use imgui::Ui;
use imnodes::{IdentifierGenerator, NodeScope};
use lifec::{Entity, Sequence, ThunkContext, World};

use crate::NodeContext;

mod single_io;
pub use single_io::SingleIO;

/// Implement trait to control node display and interaction,
///
pub trait NodeDevice {
    /// Renders the node,
    ///
    /// returns true if a button was pressed on this node,
    ///
    fn render(
        &self,
        scope: NodeScope,
        node_context: &NodeContext,
        thunk_context: &ThunkContext,
        ui: &Ui,
    ) -> Option<NodeEvent>;

    /// Handles a node event,
    ///
    fn on_event(&self, world: &World, node_event: NodeEvent);

    /// Creates a new node context,
    ///
    fn create(world: &World, sequence: &Sequence, idgen: &mut IdentifierGenerator) -> NodeContext;
}

/// Enumeration of node events that can be returned by render,
///
pub enum NodeEvent {
    /// Emitted to indicated a button on the node was pressed,
    /// Returns the name of the button, as well as the entity of node_context,
    ButtonPress { name: String, entity: Entity },
}

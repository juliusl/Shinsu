use imgui::Ui;
use imnodes::NodeScope;
use lifec::prelude::{Entity, Node};

use crate::{NodeContext, Nodes};

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
        node: &Node,
        ui: &Ui,
    ) -> Option<NodeEvent>;

    fn on_event(&mut self, events: &mut Nodes, node_event: NodeEvent);
}

/// Enumeration of node events that can be returned by render,
///
pub enum NodeEvent {
    /// Emitted to indicated a button on the node was pressed,
    /// Returns the name of the button, as well as the entity of node_context,
    ButtonPress { name: &'static str, entity: Entity },
}

impl NodeEvent {
    /// Returns a new button press,
    /// 
    pub fn button_press(name: &'static str, entity: Entity) -> Self {
        NodeEvent::ButtonPress { name, entity }
    }
}

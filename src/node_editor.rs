use atlier::system::App;
use imnodes::*;
use lifec::prelude::{Connection, Sequence};
use specs::WorldExt;
use specs::{Entity, World};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::LinkContext;
use crate::NodeContext;

use tracing::event;
use tracing::Level;

/// Index for fetching the sequence a node id represents,
///
pub type NodeIndex = HashMap<NodeId, Entity>;

/// Index for fetching the connection a link id represents
///
pub type LinkIndex = HashMap<Link, LinkId>;

/// Set for connected nodes
///
pub type ConnectedNodes = HashSet<Link>;

/// Struct for node editor state,
///
pub struct NodeEditor {
    node_index: NodeIndex,
    link_index: LinkIndex,
    connected: ConnectedNodes,
    idgen: Option<imnodes::IdentifierGenerator>,
}

impl Default for NodeEditor {
    fn default() -> Self {
        Self {
            node_index: Default::default(),
            link_index: Default::default(),
            connected: Default::default(),
            idgen: Default::default(),
        }
    }
}

impl NodeEditor {
    /// Creates a new node editor,
    ///
    pub fn new(idgen: IdentifierGenerator) -> Self {
        Self {
            idgen: Some(idgen),
            node_index: NodeIndex::default(),
            link_index: LinkIndex::default(),
            connected: HashSet::default(),
        }
    }

    /// Returns an immutable reference to the node_index,
    ///
    pub fn node_index(&self) -> &NodeIndex {
        &self.node_index
    }

    /// Returns a mutable reference to the node_index,
    ///
    pub fn node_index_mut(&mut self) -> &mut NodeIndex {
        &mut self.node_index
    }

    /// Adds a new node to represent the sequence,
    ///
    /// Returns the node context created,
    ///
    pub fn add_node(&mut self, node: Entity) -> NodeContext {
        if let Some(idgen) = self.idgen.as_mut() {
            let context = NodeContext::new(node, idgen); 
            self.node_index.insert(context.node_id(), node);
            context
        } else {
            panic!("Node editor isn't initialized")
        }
    }

    /// Adds a new link between two node contexts,
    ///
    pub fn add_link(
        &mut self,
        link: Link,
    ) -> LinkId {
        if let Some(linkid) = self.link_index.get(&link) {
            *linkid
        } else if let Some(idgen) = self.idgen.as_mut() {
            let link_id = idgen.next_link();
            self.link_index.insert(link, link_id);
            link_id
        } else {
            panic!("Node editor isn't initialized")
        }
    }

    /// Removes a link by id from the world,
    ///
    pub fn remove_link_by_id(&mut self, world: &World, link_id: LinkId) {
        let mut links = world.write_component::<LinkContext>();
        let mut sequences = world.write_component::<Sequence>();

        // if let Some(LinkContext(connection, Some(link), ..)) = self
        //     .link_index
        //     .remove(&link_id)
        //     .and_then(|d| d.owner())
        //     .and_then(|d| links.remove(d))
        // {
        //     if self.connected.remove(&link) {
        //         event!(Level::TRACE, "dropped link");
        //         // if let (Some(from), Some(to)) = connection.connection() {
        //         //     if let Some(seq) = sequences.get_mut(from) {
        //         //         event!(Level::TRACE, "disconnecting {} -/> {}", from.id(), to.id());
        //         //         *seq = seq.disconnect_by(to);
        //         //     }
        //         // }
        //     }
        // }
    }
}

impl App for NodeEditor {
    fn name() -> &'static str {
        "node_editor"
    }

    fn edit_ui(&mut self, _: &imgui::Ui) {
        
    }

    fn display_ui(&self, ui: &imgui::Ui) {
        imgui::Window::new("Nodes").build(ui, ||{
            for (node_id, entity) in self.node_index.iter() {
                ui.text(format!("{:?}: {:?}", node_id, entity));
                ui.text(format!("grid_space: pos: {:?}", node_id.get_position(CoordinateSystem::GridSpace)));
                ui.text(format!("size: {:?}", node_id.get_dimensions()));
            }
        });
    }
}

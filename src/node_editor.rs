use imnodes::*;
use lifec::prelude::{Connection, Sequence};
use specs::World;
use specs::WorldExt;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::LinkContext;
use crate::NodeContext;
use crate::NodeDevice;

use tracing::event;
use tracing::Level;

/// Index for fetching the sequence a node id represents,
///
pub type NodeIndex = HashMap<NodeId, Sequence>;

/// Index for fetching the connection a link id represents
///
pub type LinkIndex = HashMap<LinkId, Connection>;

/// Set for connected nodes
///
pub type ConnectedNodes = HashSet<Link>;

/// Struct for node editor state,
///
pub struct NodeEditor {
    node_index: NodeIndex,
    link_index: LinkIndex,
    connected: ConnectedNodes,
    idgen: imnodes::IdentifierGenerator,
}

impl NodeEditor {
    /// Creates a new node editor,
    ///
    pub fn new(idgen: IdentifierGenerator) -> Self {
        Self {
            idgen,
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
    pub fn add_node<T>(&mut self, world: &World, sequence: &Sequence) -> NodeContext
    where
        T: NodeDevice,
    {
        let context = T::create(world, sequence, &mut self.idgen);

        self.node_index
            .insert(context.node_id().expect("just generated"), sequence.clone());

        let mut nodes = world.write_component::<NodeContext>();
        if let Some(start) = sequence.peek() {
            nodes
                .insert(start, context.clone())
                .expect("should be able to insert context");
        }

        context
    }

    /// Adds a new link between two node contexts,
    ///
    /// TODO: Will add support for multi i/o by passing in start/end pins
    ///
    pub fn add_link(
        &mut self,
        world: &World,
        from: &Sequence,
        to: &Sequence,
        connection: Connection, 
        link: Link,
    ) -> Option<Connection> {
        // let mut connection = from.connect(&to);

        if let (Some(from), Some(_)) = (from.peek(), to.peek()) {
            if self.connected.insert(link) {
                let link_entity = world.entities().create();

                // sets an owner for the connection
                // this makes it easier to clean up dropped connections
                // connection.set_owner(link_entity);

                let context =
                    LinkContext(connection.clone(), Some(link), Some(self.idgen.next_link()));

                let mut links = world.write_component::<LinkContext>();
                match links.insert(link_entity, context.clone()) {
                    Ok(_) => {
                        if let Some(link_id) = context.link_id() {
                            self.link_index.insert(link_id, connection.clone());
                        }
                    }
                    Err(_) => {}
                }

                let mut connections = world.write_component::<Connection>();
                match connections.insert(from, connection.clone()) {
                    Ok(_) => {
                        return Some(connection);
                    }
                    Err(_) => {}
                }
            } else {
                event!(Level::TRACE, "Already connected");
            }
        }

        None
    }

    /// Removes a link by id from the world,
    ///
    pub fn remove_link_by_id(&mut self, world: &World, link_id: LinkId) {
        let mut links = world.write_component::<LinkContext>();
        let mut sequences = world.write_component::<Sequence>();

        if let Some(LinkContext(connection, Some(link), ..)) = self
            .link_index
            .remove(&link_id)
            .and_then(|d| d.owner())
            .and_then(|d| links.remove(d))
        {
            if self.connected.remove(&link) {
                event!(Level::TRACE, "dropped link");
                if let (Some(from), Some(to)) = connection.connection() {
                    if let Some(seq) = sequences.get_mut(from) {
                        event!(Level::TRACE, "disconnecting {} -/> {}", from.id(), to.id());
                        *seq = seq.disconnect_by(to);
                    }
                }
            }
        }
    }
}

use imnodes::*;
use lifec::Connection;
use lifec::Sequence;
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

/// Struct for node editor state,
///
pub struct NodeEditor
{
    node_index: NodeIndex,
    link_index: LinkIndex,
    connected: HashSet<NodeId>,
    idgen: imnodes::IdentifierGenerator,
}

impl NodeEditor
{
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
    pub fn add_node<T>(&mut self, world: &World, sequence: &Sequence) 
    where
        T: NodeDevice
    {
        let context = T::create(world, sequence, &mut self.idgen);

        self.node_index.insert(context.node_id().expect("just generated"), sequence.clone());

        let mut nodes = world.write_component::<NodeContext>();
        if let Some(start) = sequence.peek() {
            match nodes.insert(start, context) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    /// Adds a new link between two node contexts,
    /// 
    pub fn add_link(
        &mut self,
        world: &World,
        from: &Sequence,
        to: &Sequence,
    ) -> Option<Connection> {
        let nodes = world.read_component::<NodeContext>();

        let mut connection = from.connect(&to);

        if let (Some(from), Some(to)) = (from.peek(), to.peek()) {
            if let (
                Some(NodeContext(_, Some(start_node), Some(_), Some(start_pin), ..)),
                Some(NodeContext(_, Some(end_node), Some(end_pin), Some(_), ..)),
            ) = (nodes.get(from), nodes.get(to))
            {
                // TODO currently this is a limitation, to having only 1 node connected to the output
                if self.connected.insert(*start_node) {
                    let start_node = *start_node;
                    let end_node = *end_node;
                    let start_pin = *start_pin;
                    let end_pin = *end_pin;

                    let link = Link {
                        start_node,
                        end_node,
                        start_pin,
                        end_pin,
                        craeated_from_snap: true,
                    };
                    let link_entity = world.entities().create();

                    // sets an owner for the connection
                    // this makes it easier to clean up dropped connections
                    connection.set_owner(link_entity);

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
        }

        None
    }

    /// Removes a link by id from the world,
    /// 
    pub fn remove_link_by_id(&mut self, world: &World, link_id: LinkId) {
        let mut links = world.write_component::<LinkContext>();

        if let Some(drop) = self.link_index.remove(&link_id) {
            if let Some(drop) = drop.owner() {
                if let Some(dropped) = links.remove(drop) {
                    if let LinkContext(_, Some(link), ..) = dropped {
                        let Link { start_node, .. } = link;

                        if self.connected.remove(&start_node) {
                            event!(Level::TRACE, "dropped link {:?}", drop);
                        }
                    }
                }
            }
        }
    }
}


use lifec::prelude::Events;
use lifec::prelude::Node;
use specs::prelude::*;
use specs::SystemData;
use tracing::event;
use tracing::Level;

use crate::LinkContext;
use crate::NodeContext;
use crate::NodeEditor;

///
///
#[derive(SystemData)]
pub struct Nodes<'a> {
    node_editor: Write<'a, NodeEditor>,
    events: Events<'a>,
    nodes: WriteStorage<'a, NodeContext>,
}

impl<'a> Nodes<'a> {
    /// Returns a mutable reference to events system data,
    /// 
    pub fn events(&self) -> &Events<'a> {
        &self.events
    }

    /// Scans nodes and creates node contexts for them,
    ///
    pub fn scan_nodes(&mut self) -> (Vec<(NodeContext, Node)>, Vec<LinkContext>) {
        let mut nodes = vec![];
        let mut links = vec![];
        for (col, node) in self.events.nodes().iter().enumerate() {
            match node.status {
                lifec::prelude::NodeStatus::Event(event_status) if !node.is_adhoc() && !node.is_spawned() => {
                    let entity = event_status.entity();

                    // Initialize node context
                    //
                    if !self.nodes.contains(entity) {
                        let mut context = self.node_editor.add_node(entity);
                        context.col = col + 1;

                        self.nodes
                            .insert(entity, context.clone())
                            .expect("should be able to insert node context");

                        nodes.push((context, node.clone()));
                    } else if let Some(context) = self.nodes.get(entity) {
                        nodes.push((context.clone(), node.clone()));
                    }
                }
                _ => {}
            }

            let mut assign_rows = vec![];
            if let Some(connection) = node.connection.as_ref() {
                for (row, (from, to)) in connection
                    .connections()
                    .map(|(from, to)| (self.nodes.get(*from), self.nodes.get(*to)))
                    .enumerate()
                {
                    if let (Some(from), Some(to)) = (from, to) {
                        let link = from.link(to, false);

                        let link_id = self.node_editor.get_link(link);

                        links.push(LinkContext { link, link_id });

                        assign_rows.push((from.entity, row));
                    }
                }
            }

            for (from, row) in assign_rows.iter() {
                if let Some(from) = self.nodes.get_mut(*from) {
                    from.row = *row;
                }
            }
        }

        (nodes, links)
    }

    /// Rearrange nodes by links,
    ///
    pub fn rearrange(&mut self) {
        let (nodes, _) = self.scan_nodes();

        for (
            NodeContext {
                node_id, row, col, ..
            },
            _,
        ) in nodes.iter()
        {
            let x = *col as f32 * 250.0 - 100.0;
            let y = *row as f32 * 200.0 + 200.0;
            event!(Level::INFO, "Rearranging, {:?} {x}, {y}", node_id);
            node_id.set_position(x, y, imnodes::CoordinateSystem::GridSpace);
        }
    }

    /// Returns the node editor,
    ///
    pub fn node_editor(&self) -> &NodeEditor {
        &self.node_editor
    }
}

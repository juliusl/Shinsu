use imnodes::*;
use lifec::plugins::ThunkContext;
use lifec::Connection;
use lifec::Extension;
use lifec::Sequence;
use specs::Entities;
use specs::Join;
use specs::ReadStorage;
use specs::RunNow;
use specs::System;
use specs::World;
use specs::WorldExt;
use specs::WriteStorage;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::LinkContext;
use crate::NodeContext;
use crate::NodeDevice;

/// Struct for node editor state,
///
pub struct NodeEditor<T>
where
    T: NodeDevice,
{
    dropping: Vec<LinkId>,
    connecting: Vec<Link>,
    creating: Vec<Sequence>,
    node_index: HashMap<NodeId, Sequence>,
    link_index: HashMap<LinkId, Connection>,
    node_device: T,
    editor_context: imnodes::EditorContext,
    idgen: imnodes::IdentifierGenerator,
    connected: HashSet<NodeId>,
    _imnodes: imnodes::Context,
}

impl<T> NodeEditor<T>
where
    T: NodeDevice,
{
    /// Sets the node_device for the editor, 
    /// 
    /// The node device is stateless, and only renders nodes,
    /// 
    pub fn set_node_device(&mut self, node_device: T) {
        self.node_device = node_device;
    }

    /// Adds a new node to represent the sequence,
    /// 
    pub fn add_node(&mut self, world: &World, sequence: &Sequence) {
        let context = NodeContext(
            sequence.clone(),
            Some(self.idgen.next_node()),
            Some(self.idgen.next_input_pin()),
            Some(self.idgen.next_output_pin()),
            Some(self.idgen.next_attribute()),
        );

        self.node_index.insert(context.node_id().expect("just generated"), sequence.clone());

        let mut nodes = world.write_component::<NodeContext>();
        if let Some(start) = sequence.peek() {
            match nodes.insert(start, context) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    /// Adds a new link that represents
    pub fn add_link(
        &mut self,
        app_world: &World,
        from: &Sequence,
        to: &Sequence,
    ) -> Option<Connection> {
        let nodes = app_world.read_component::<NodeContext>();

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
                    let link_entity = app_world.entities().create();

                    // sets an owner for the connection
                    // this makes it easier to clean up dropped connections
                    connection.set_owner(link_entity);

                    let context =
                        LinkContext(connection.clone(), Some(link), Some(self.idgen.next_link()));

                    let mut links = app_world.write_component::<LinkContext>();
                    match links.insert(link_entity, context.clone()) {
                        Ok(_) => {
                            if let Some(link_id) = context.link_id() {
                                self.link_index.insert(link_id, connection.clone());
                            }
                        }
                        Err(_) => {}
                    }

                    let mut connections = app_world.write_component::<Connection>();
                    match connections.insert(from, connection.clone()) {
                        Ok(_) => {
                            return Some(connection);
                        }
                        Err(_) => {}
                    }
                } else {
                    eprintln!("Already connected");
                }
            }
        }

        None
    }

    /// Removes a link by id from the world
    pub fn remove_link_by_id(&mut self, app_world: &World, link_id: LinkId) {
        let mut links = app_world.write_component::<LinkContext>();

        if let Some(drop) = self.link_index.remove(&link_id) {
            if let Some(drop) = drop.owner() {
                if let Some(dropped) = links.remove(drop) {
                    if let LinkContext(_, Some(link), ..) = dropped {
                        let Link { start_node, .. } = link;

                        if self.connected.remove(&start_node) {
                            eprintln!("dropped link {:?}", drop);
                        }
                    }
                }
            }
        }
    }
}

impl<T> Default for NodeEditor<T>
where
    T: NodeDevice + Default,
{
    fn default() -> Self {
        let _imnodes = imnodes::Context::new();
        let editor_context = _imnodes.create_editor();
        let idgen = editor_context.new_identifier_generator();
        Self {
            editor_context,
            idgen,
            _imnodes,
            connected: HashSet::default(),
            node_index: HashMap::default(),
            link_index: HashMap::default(),
            creating: vec![],
            connecting: vec![],
            dropping: vec![],
            node_device: T::default(),
        }
    }
}

impl<T> Extension for NodeEditor<T>
where
    T: NodeDevice,
{
    fn configure_app_world(world: &mut World) {
        world.register::<NodeContext>();
        world.register::<LinkContext>();
        world.register::<ThunkContext>();
    }

    fn configure_app_systems(dispatcher: &mut specs::DispatcherBuilder) {
        // linking system to set sequence cursors based on connections
        dispatcher.add(Linker {}, "shinsu/linker", &[]);
    }

    fn on_ui(&'_ mut self, app_world: &specs::World, ui: &'_ imgui::Ui<'_>) {
        let nodes = app_world.read_component::<NodeContext>();
        let links = app_world.read_component::<LinkContext>();
        let thunks = app_world.write_component::<ThunkContext>();

        let detatch = self
            .editor_context
            .push(AttributeFlag::EnableLinkDetachWithDragClick);

        let outer_scope = editor(&mut self.editor_context, |mut editor_scope| {
            editor_scope.add_mini_map(MiniMapLocation::BottomRight);

            for node in nodes.join() {
                if let NodeContext(sequence, Some(node_id), ..) = node {
                    editor_scope.add_node(*node_id, |node_scope| {
                        if let Some(from) = sequence.last() {
                            if let Some(tc) = thunks.get(from) {
                                self.node_device.render(node_scope, node, tc, ui);
                            }
                        }
                    });
                }
            }

            for link in links.join() {
                if let LinkContext(
                    ..,
                    Some(Link {
                        start_pin, end_pin, ..
                    }),
                    Some(link_id),
                ) = link
                {
                    editor_scope.add_link(*link_id, *end_pin, *start_pin);
                }
            }
        });

        if let Some(link) = outer_scope.links_created() {
            self.connecting.push(link);
        }

        if let Some(dropped) = outer_scope.get_dropped_link() {
            self.dropping.push(dropped);
        }

        detatch.pop();
    }

    fn on_run(&'_ mut self, app_world: &specs::World) {
        while let Some(link) = self.connecting.pop() {
            let Link {
                start_node,
                end_node,
                ..
            } = link;
            let from = self
                .node_index
                .get(&start_node)
                .and_then(|s| Some(s.clone()));
            let to = self.node_index.get(&end_node).and_then(|s| Some(s.clone()));

            if let (Some(from), Some(to)) = (from, to) {
                self.add_link(app_world, &from, &to);
            }
        }

        while let Some(drop) = self.dropping.pop() {
            self.remove_link_by_id(app_world, drop);
        }

        while let Some(create) = self.creating.pop() {
            self.add_node(app_world, &create);
        }

        self.run_now(app_world);
    }
}

impl<'a, T> System<'a> for NodeEditor<T>
where
    T: NodeDevice,
{
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Sequence>,
        ReadStorage<'a, Connection>,
        ReadStorage<'a, NodeContext>,
    );

    fn run(&mut self, (entities, sequences, connections, nodes): Self::SystemData) {
        for (entity, sequence, _connection, node) in
            (&entities, &sequences, &connections, nodes.maybe()).join()
        {
            if let None = node {
                let mut clone = sequence.clone();
                clone.push(entity);
                self.creating.push(clone);
            }
        }
    }
}

struct Linker;

impl<'a> System<'a> for Linker {
    type SystemData = (WriteStorage<'a, Connection>, WriteStorage<'a, Sequence>);

    fn run(&mut self, (mut connections, mut sequences): Self::SystemData) {
        for (connection, sequence) in (&mut connections, &mut sequences).join() {
            if let (_, Some(to)) = connection.connection() {
                sequence.set_cursor(to);
            }
        }
    }
}

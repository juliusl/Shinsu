use imgui::Ui;
use imnodes::*;
use lifec::plugins::CancelThunk;
use lifec::plugins::Connection;
use lifec::plugins::Event;
use lifec::plugins::Sequence;
use lifec::plugins::ThunkContext;
use lifec::Extension;
use specs::Component;
use specs::DenseVecStorage;
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

/// This component renders a node to the editor
#[derive(Component, Clone, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct NodeContext(
    pub Sequence,
    pub Option<NodeId>,
    pub Option<InputPinId>,
    pub Option<InputPinId>,
    pub Option<OutputPinId>,
    pub Option<AttributeId>,
);

impl NodeContext {
    pub fn node_id(&self) -> Option<NodeId> {
        self.1
    }

    pub fn fork_pin(&self) -> Option<InputPinId> {
        self.3
    }
}

/// This component, renders a link to the editor
#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct LinkContext(pub Connection, pub Option<Link>, pub Option<LinkId>);

impl LinkContext {
    pub fn link_id(&self) -> Option<LinkId> {
        self.2.clone()
    }
}

/// Function for displaying ui on the node
pub type NodeUI = fn(NodeScope, &NodeContext, &ThunkContext, &Ui) -> bool;

/// Extension w/ node editor
pub struct NodeEditor {
    dropping: Vec<LinkId>,
    connecting: Vec<Link>,
    creating: Vec<Sequence>,
    node_index: HashMap<NodeId, Sequence>,
    link_index: HashMap<LinkId, Connection>,
    node_ui: NodeUI,
    editor_context: imnodes::EditorContext,
    idgen: imnodes::IdentifierGenerator,
    _imnodes: imnodes::Context,
    _connected: HashSet<NodeId>,
    _fork_pin: HashSet<InputPinId>,
}

impl NodeEditor {
    /// Set's the ui to display on each node
    /// By default only input/output are shown
    pub fn set_ui(&mut self, ui_fn: NodeUI) {
        self.node_ui = ui_fn;
    }

    /// Adds a new node to represent the sequence
    pub fn add_node(&mut self, app_world: &World, sequence: &Sequence) {
        let context = NodeContext(
            sequence.clone(),
            Some(self.idgen.next_node()),
            Some(self.idgen.next_input_pin()),
            Some(self.idgen.next_input_pin()),
            Some(self.idgen.next_output_pin()),
            Some(self.idgen.next_attribute()),
        );

        let fork_pin = context.fork_pin().expect("just added");
        self._fork_pin.insert(fork_pin);

        self.node_index
            .insert(context.node_id().expect("just generated"), sequence.clone());

        let mut nodes = app_world.write_component::<NodeContext>();
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
        fork: bool,
    ) -> Option<Connection> {
        let nodes = app_world.read_component::<NodeContext>();
        let to = {
            if fork {
                if let Some(fork) = to.fork() {
                    fork
                } else {
                    to.clone()
                }
            } else {
                to.clone()
            }
        };

        let mut connection = from.connect(&to);
        if let (Some(from), Some(to)) = (from.peek(), to.peek()) {
            if let (
                Some(NodeContext(_, Some(start_node), Some(_), Some(_), Some(start_pin), ..)),
                Some(NodeContext(_, Some(end_node), Some(end_pin), Some(fork_pin), Some(_), ..)),
            ) = (nodes.get(from), nodes.get(to))
            {
                // TODO currently this is a limitation, to having only 1 node connected to the output
                if self._connected.insert(*start_node) {
                    let start_node = *start_node;
                    let end_node = *end_node;
                    let start_pin = *start_pin;
                    let end_pin = {
                        if fork {
                            fork_pin
                        } else {
                            end_pin
                        }
                    };
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

                        if self._connected.remove(&start_node) {
                            eprintln!("dropped link {:?}", drop);
                        }
                    }
                }
            }

            if let (Some(from), Some(_)) = drop.connection() {
                match app_world
                    .write_component::<Connection>()
                    .insert(from, Connection::default())
                {
                    Ok(_) => match app_world.write_component::<Sequence>().get_mut(from) {
                        Some(sequence) => *sequence = sequence.disconnect(),
                        None => {}
                    },
                    Err(_) => {}
                }
            }
        }
    }
}

impl Default for NodeEditor {
    fn default() -> Self {
        let _imnodes = imnodes::Context::new();
        let editor_context = _imnodes.create_editor();
        let idgen = editor_context.new_identifier_generator();
        Self {
            editor_context,
            idgen,
            _imnodes,
            _connected: HashSet::default(),
            _fork_pin: HashSet::default(),
            node_index: HashMap::default(),
            link_index: HashMap::default(),
            creating: vec![],
            connecting: vec![],
            dropping: vec![],
            node_ui: |mut scope, nc, tc, ui| {
                if let Some(node_title) = tc.as_ref().find_text("node_title") {
                    let entity = tc.entity.and_then(|e| Some(e.id())).unwrap_or(0);
                    scope.add_titlebar(|| {
                        ui.text(format!("{entity} {node_title}"));
                    });
                    let thunk_symbol = tc
                        .block
                        .as_ref()
                        .find_text("thunk_symbol")
                        .unwrap_or("entity".to_string());
                    let mut node_width = 75.0;
                    if thunk_symbol.len() > 24 {
                        node_width = 150.0;
                    }

                    if let NodeContext(
                        ..,
                        Some(input_pin),
                        Some(fork_pin),
                        Some(output_pin),
                        Some(attribute_id),
                    ) = nc
                    {
                        scope.attribute(*attribute_id, || {
                            ui.text(format!("{} {}", tc.block.block_name, thunk_symbol));
                        });
                        scope.add_input(*input_pin, PinShape::Circle, || {
                            let label = tc
                                .as_ref()
                                .find_text("node_input_label")
                                .unwrap_or("start".to_string());
                            ui.text(label);
                        });

                        ui.same_line();
                        scope.add_output(*output_pin, PinShape::CircleFilled, || {
                            ui.same_line();
                            ui.set_next_item_width(node_width);
                            ui.label_text("cursor", "");
                        });

                        if let Some(true) = tc.as_ref().is_enabled("enable_fork") {
                            scope.add_input(*fork_pin, PinShape::Quad, || {
                                let label = tc
                                    .as_ref()
                                    .find_text("node_fork_input_label")
                                    .unwrap_or("fork".to_string());
                                ui.text(label);
                            });
                        }
                    }

                    return if thunk_symbol.contains("Running") {
                        ui.button("cancel")
                    } else {
                        ui.button("start")
                    };
                }

                false
            },
        }
    }
}

impl Extension for NodeEditor {
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
        let thunks = app_world.read_component::<ThunkContext>();
        let mut events = app_world.write_component::<Event>();
        let mut cancel_events = app_world.write_component::<CancelThunk>();

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
                                if (self.node_ui)(node_scope, node, tc, ui) {
                                    if let Some(event) = events.get_mut(from) {
                                        if event.is_running() {
                                            if let Some(cancel_thunk) = cancel_events.remove(from) {
                                                cancel_thunk.0.send(()).ok();
                                            }
                                        } else {
                                            event.fire(tc.clone());
                                        }
                                    }
                                }
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
                end_pin,
                ..
            } = link;
            let from = self
                .node_index
                .get(&start_node)
                .and_then(|s| Some(s.clone()));
            let to = self.node_index.get(&end_node).and_then(|s| Some(s.clone()));

            let fork = self._fork_pin.contains(&end_pin);

            if let (Some(from), Some(to)) = (from, to) {
                self.add_link(app_world, &from, &to, fork);
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

impl<'a> System<'a> for NodeEditor {
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
    type SystemData = (ReadStorage<'a, Connection>, WriteStorage<'a, Sequence>);

    fn run(&mut self, (connections, mut sequences): Self::SystemData) {
        for (connection, sequence) in (&connections, &mut sequences).join() {
            if let (_, Some(to)) = connection.connection() {
                sequence.set_cursor(to);
            }
        }
    }
}

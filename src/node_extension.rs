use imnodes::{editor, AttributeFlag, Link, LinkId, MiniMapLocation};
use lifec::{
    AttributeGraph, Block, Connection, Entities, Extension, Interpreter, Join, ReadStorage,
    Sequence, System, ThunkContext, World, WorldExt, WriteStorage,
};
use specs::RunNow;

use crate::{LinkContext, NodeContext, NodeDevice, NodeEditor};

/// Extension/Interpreter that implements a node editor,
///
/// Entities w/ the sequence component are candidates for being represented as a node,
///
pub struct NodeExtension<T>
where
    T: NodeDevice,
{
    node_device: T,
    dropping: Vec<LinkId>,
    connecting: Vec<Link>,
    creating: Vec<Sequence>,
    editor_context: imnodes::EditorContext,
    _imnodes: imnodes::Context,
}

impl<T> Interpreter for NodeExtension<T>
where
    T: NodeDevice,
{
    fn initialize(&self, world: &mut World) {
        let idgen = self.editor_context.new_identifier_generator();
        let node_editor = NodeEditor::new(idgen);

        world.insert(node_editor);
    }

    fn interpret(&self, _world: &World, _block: &lifec::Block) {
        // TODO
    }
}

impl<T> NodeExtension<T>
where
    T: NodeDevice,
{
    /// Creates a new NodeExtension and inserts a node editor into the world,
    ///
    pub fn new(node_device: T) -> Self {
        let _imnodes = imnodes::Context::new();
        let editor_context = _imnodes.create_editor();
        let node_extension = Self {
            editor_context,
            _imnodes,
            creating: vec![],
            connecting: vec![],
            dropping: vec![],
            node_device,
        };

        node_extension
    }

    /// Sets the node_device for the editor,
    ///
    /// The node device is stateless, and only renders nodes,
    ///
    pub fn set_node_device(&mut self, node_device: T) {
        self.node_device = node_device;
    }
}

impl<T> From<T> for NodeExtension<T>
where
    T: NodeDevice,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Extension for NodeExtension<T>
where
    T: NodeDevice,
{
    fn configure_app_world(world: &mut World) {
        world.register::<NodeContext>();
        world.register::<LinkContext>();
        world.register::<ThunkContext>();
    }

    fn configure_app_systems(dispatcher: &mut specs::DispatcherBuilder) {
        dispatcher.add(Linker {}, "", &[]);
    }

    fn on_ui(&'_ mut self, app_world: &specs::World, ui: &'_ imgui::Ui<'_>) {
        let nodes = app_world.read_component::<NodeContext>();
        let links = app_world.read_component::<LinkContext>();
        let thunks = app_world.read_component::<ThunkContext>();
        let graphs = app_world.read_component::<AttributeGraph>();
        let blocks = app_world.read_component::<Block>();

        let detatch = self
            .editor_context
            .push(AttributeFlag::EnableLinkDetachWithDragClick);

        let outer_scope = editor(&mut self.editor_context, |mut editor_scope| {
            editor_scope.add_mini_map(MiniMapLocation::BottomRight);

            for node in nodes.join() {
                if let NodeContext(sequence, Some(node_id), ..) = node {
                    editor_scope.add_node(*node_id, |node_scope| {
                        if let Some(from) = sequence.last() {
                            if let (Some(tc), Some(graph), Some(block)) =
                                (thunks.get(from), graphs.get(from), blocks.get(from))
                            {
                                // TODO -- This could probably be improved
                                self.node_device.render(
                                    node_scope,
                                    node,
                                    &tc.with_state(graph.clone()).with_block(block),
                                    ui,
                                );
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
        let mut node_editor = app_world.write_resource::<NodeEditor>();

        while let Some(link) = self.connecting.pop() {
            let Link {
                start_node,
                end_node,
                ..
            } = link;
            let from = node_editor
                .node_index()
                .get(&start_node)
                .and_then(|s| Some(s.clone()));
            let to = node_editor
                .node_index()
                .get(&end_node)
                .and_then(|s| Some(s.clone()));

            if let (Some(from), Some(to)) = (from, to) {
                node_editor.add_link(app_world, &from, &to);
            }
        }

        while let Some(drop) = self.dropping.pop() {
            node_editor.remove_link_by_id(app_world, drop);
        }

        while let Some(create) = self.creating.pop() {
            node_editor.add_node::<T>(app_world, &create);
        }

        self.run_now(app_world);
    }
}

impl<'a, T> System<'a> for NodeExtension<T>
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
        // Iterate over entities with a sequence/connection but no node context
        for (entity, sequence, _, node) in
            (&entities, &sequences, &connections, nodes.maybe()).join()
        {
            if let None = node {
                // The first entity in a sequence will have the connection/sequence components
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

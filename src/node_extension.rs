use atlier::system::App;
use imgui::{StyleVar, Window};
use imnodes::{editor, AttributeFlag, Link, MiniMapLocation};
use lifec::{
    prelude::{
        Block, Extension, Interpreter, World, WorldExt,
    },
};
use crate::{LinkContext, NodeContext, NodeDevice, NodeEditor, Nodes, SingleIO};

/// Extension/Interpreter that implements a node editor,
///
/// Entities w/ the sequence component are candidates for being represented as a node,
///
pub struct NodeExtension<T = SingleIO>
where
    T: NodeDevice,
{
    node_device: T,
    // dropping: Vec<LinkId>,
    // connecting: Vec<Link>,
    // creating: Vec<Sequence>,
    editor_context: imnodes::EditorContext,
    _imnodes: imnodes::Context,
    opened: bool,
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

    fn interpret(&self, _world: &World, _block: &Block) {}
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
            node_device,
            opened: false,
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
    }

    fn on_ui(&'_ mut self, world: &specs::World, ui: &'_ imgui::Ui<'_>) {
        let mut nodes_data = world.system_data::<Nodes>();

        Window::new("Nodes")
            .size([1600.0, 900.0], imgui::Condition::Appearing)
            .build(ui, || {
                let frame_padding = ui.push_style_var(StyleVar::FramePadding([8.0, 5.0]));
                let detatch = self
                    .editor_context
                    .push(AttributeFlag::EnableLinkDetachWithDragClick);

                if ui.button("Rearrange nodes") {
                    nodes_data.rearrange();
                }

                ui.spacing();
                ui.separator();

                let _ = editor(&mut self.editor_context, |mut editor_scope| {
                    if !self.opened {
                        self.opened = true;
                        nodes_data.rearrange();
                    }

                    editor_scope.add_mini_map(MiniMapLocation::BottomRight);
                    let (nodes, links) = nodes_data.scan_nodes();

                    for (node_context, node) in nodes {
                        let NodeContext { node_id, .. } = node_context;

                        editor_scope.add_node(node_id, |node_scope| {
                            if let Some(event) = self
                                .node_device
                                .render(node_scope, &node_context, &node, ui)
                                .take()
                            {
                                self.node_device.on_event(&mut nodes_data, event);
                            }
                        });
                    }

                    for LinkContext {
                        link:
                            Link {
                                start_pin, end_pin, ..
                            },
                        link_id,
                        ..
                    } in links
                    {
                        editor_scope.add_link(link_id, end_pin, start_pin);
                    }
                });

                // if let Some(link) = outer_scope.links_created() {
                //     self.connecting.push(link);
                // }

                // if let Some(dropped) = outer_scope.get_dropped_link() {
                //     self.dropping.push(dropped);
                // }

                detatch.pop();
                frame_padding.end();
            });

        nodes_data.node_editor().display_ui(ui);
    }
}

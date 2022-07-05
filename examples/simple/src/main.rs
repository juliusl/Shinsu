
use imgui::Window;
use lifec::{open, Runtime, editor::*, plugins::*};
use shinsu::NodeEditor;

fn main() {
    let mut runtime = Runtime::new(Project::runmd().unwrap_or_default());
    runtime.install::<Call, Timer>();
    runtime.install::<Call,OpenFile>();
    runtime.install::<Call, WriteFile>();

    runtime.add_config(Config("test", |tc| {
        tc.block.block_name = unique_title("demo");
        let block_name = tc.block.block_name.to_string();
        tc.as_mut()
            .with_text("node_title", block_name)
            .add_int_attr("duration", 2);
    }));

    open(
        "shinsu demo", 
        Runtime::default(),
        Demo(NodeEditor::default(), RuntimeEditor::new(runtime)),
    );
}

#[derive(Default)]
struct Demo(NodeEditor, RuntimeEditor);


impl Extension for Demo {
    fn configure_app_world(world: &mut lifec::plugins::World) {
        NodeEditor::configure_app_world(world);
        RuntimeEditor::configure_app_world(world);
    }

    fn configure_app_systems(dispatcher: &mut DispatcherBuilder) {
        NodeEditor::configure_app_systems(dispatcher);
        RuntimeEditor::configure_app_systems(dispatcher);
    }

    fn on_ui(&'_ mut self, app_world: &lifec::plugins::World, ui: &'_ imgui::Ui<'_>) {
        Window::new("demo").build(ui, ||{
            if ui.button("create sequence") {
                let runtime = &self.1.runtime();
    
                if let Some(first) = runtime.create_engine::<Call>(
                    app_world, 
                    "from"
                ) {
                    // To enable in the node editor, add the connection component
                    app_world.write_component::<Connection>()
                        .insert(first, Connection::default()).ok();
                }

                if let Some(first) = runtime.create_engine::<Call>(
                    app_world, 
                    "to"
                ) {
                    // To enable in the node editor, add the connection component
                    app_world.write_component::<Connection>()
                        .insert(first, Connection::default()).ok();
                }
            }
        });

        self.0.on_ui(app_world, ui);
        self.1.on_ui(app_world, ui);
    }

    fn on_run(&'_ mut self, app_world: &lifec::plugins::World) {
        self.0.on_run(app_world);
        self.1.on_run(app_world);
    }
}

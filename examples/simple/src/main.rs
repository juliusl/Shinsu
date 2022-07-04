
use imgui::Window;
use lifec::{open, Runtime, editor::*, plugins::*};
use shinsu::NodeEditor;

fn main() {
    let project = Project::default();

    let mut runtime = Runtime::new(
    project.with_block("demo", "call", |c|{
            c.define("a_timer", "timer").edit_as(Value::TextBuffer("test".to_string()));
            c.define("b_timer", "timer").edit_as(Value::TextBuffer("test".to_string()));
    }));

    runtime.install::<Call, Timer>();

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
        Demo(NodeEditor::default(), RuntimeEditor::new(runtime), vec![]),
    );
}

#[derive(Default)]
struct Demo(NodeEditor, RuntimeEditor, Vec<Entity>);


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
                    "demo"
                ) {
                    self.2.push(first);
                }
            }
        });

        self.0.on_ui(app_world, ui);
        self.1.on_ui(app_world, ui);
    }

    fn on_run(&'_ mut self, app_world: &lifec::plugins::World) {
        self.0.on_run(app_world);
        self.1.on_run(app_world);

        let sequences = app_world.read_component::<Sequence>();
        while let Some(node) = self.2.pop() {
            if let Some(sequence) = sequences.get(node) {
                let mut clone = sequence.clone(); 
                clone.push(node);

                eprintln!("{:#?}", clone);

                self.0.add_node(app_world, &clone);
            }
        }
    }
}

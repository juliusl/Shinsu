use imgui::Ui;
use imnodes::PinShape;
use lifec::{AttributeIndex, ThunkContext};

use crate::{NodeContext, NodeDevice};

/// Node device implementation that is a single input/output,
///
#[derive(Default)]
pub struct SingleIO;

impl NodeDevice for SingleIO {
    fn render(&self, mut scope: imnodes::NodeScope, nc: &NodeContext, tc: &ThunkContext, ui: &Ui) {
        if let Some(node_title) = tc.search().find_symbol("node_title") {
            scope.add_titlebar(|| {
                ui.text(node_title);
            });
            let thunk_symbol = tc
                .state()
                .find_symbol("plugin_symbol")
                .unwrap_or("entity".to_string());
            let mut node_width = 75.0;
            if thunk_symbol.len() > 24 {
                node_width = 150.0;
            }

            if let NodeContext(.., Some(input_pin), Some(output_pin), Some(attribute_id)) = nc {
                scope.attribute(*attribute_id, || {
                    ui.text(format!("{} {}", tc.block().name(), thunk_symbol));
                });
                scope.add_input(*input_pin, PinShape::Circle, || {
                    let label = tc
                        .search()
                        .find_symbol("node_input_label")
                        .unwrap_or("start".to_string());
                    ui.text(label);
                });

                ui.same_line();
                scope.add_output(*output_pin, PinShape::CircleFilled, || {
                    ui.same_line();
                    ui.set_next_item_width(node_width);
                    ui.label_text("cursor", "");
                })
            }
        }
    }

    fn create (
        _: &lifec::World,
        sequence: &lifec::Sequence,
        idgen: &mut imnodes::IdentifierGenerator,
    ) -> NodeContext {
        NodeContext(
            sequence.clone(),
            Some(idgen.next_node()),
            Some(idgen.next_input_pin()),
            Some(idgen.next_output_pin()),
            Some(idgen.next_attribute()),
        )
    }
}

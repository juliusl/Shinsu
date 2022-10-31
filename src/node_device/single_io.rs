use std::collections::BTreeSet;

use imgui::Ui;
use imnodes::PinShape;
use lifec::prelude::{AttributeIndex, Node, Sequence, ThunkContext};
use specs::World;

use crate::{NodeContext, NodeDevice};

use super::NodeEvent;

/// Node device implementation that is a single input/output,
///
/// Used for setting a cursor on sequences,
///
#[derive(Default)]
pub struct SingleIO;

impl NodeDevice for SingleIO {
    fn render(
        &self,
        mut scope: imnodes::NodeScope,
        nc: &NodeContext,
        node: &Node,
        ui: &Ui,
    ) -> Option<NodeEvent> {
        if let Some(state) = node.appendix.state(&nc.entity) {
            let name = node.appendix.name(&nc.entity).unwrap_or("<unnamed>");
            scope.add_titlebar(|| {
                ui.text(format!("{}.{name}", state.control_symbol));
            });

            let node_width = 75.0;
            // if thunk_symbol.len() > 24 {
            //     node_width = 150.0;
            // }

            // Render sequence config
            let NodeContext {
                input_pin_id,
                output_pin_id,
                attribute_id,
                ..
            } = nc;
            {
                scope.attribute(*attribute_id, || match node.status {
                    lifec::prelude::NodeStatus::Event(event_status) => {
                        ui.text(format!("{event_status}"));
                    }
                    _ => ui.text("node"),
                });

                scope.add_input(*input_pin_id, PinShape::Circle, || {
                    // let label = tc
                    //     .search()
                    //     .find_symbol("node_input_label")
                    //     .unwrap_or("start".to_string());

                    if let Some(transition) = node.transition.as_ref() {
                        ui.text(format!("{:?}", transition));
                    } else {
                        ui.text("input");
                    }
                });

                ui.same_line();
                scope.add_output(*output_pin_id, PinShape::CircleFilled, || {
                    ui.same_line();
                    ui.set_next_item_width(node_width);
                    ui.label_text("Cursor", "");
                });
            }
        }

        None
    }

    fn on_event(&mut self, world: &World, node_event: NodeEvent) {
        todo!()
    }
}

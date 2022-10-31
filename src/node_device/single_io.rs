use imgui::{StyleVar, Ui};
use imnodes::PinShape;
use lifec::prelude::{EventStatus, Node, NodeCommand, NodeStatus};

use crate::{NodeContext, NodeDevice, Nodes};

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
                entity,
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

                let frame_padding = ui.push_style_var(StyleVar::FramePadding([8.0, 5.0]));
                match node.status {
                    NodeStatus::Event(event_status) => match event_status {
                        EventStatus::Inactive(_) => {
                            ui.spacing();
                            if ui.button("Start") {
                                return Some(NodeEvent::button_press("Start", *entity));
                            }

                            ui.same_line();
                            if ui.button("Pause") {
                                return Some(NodeEvent::button_press("Pause", *entity));
                            }
                        }
                        EventStatus::InProgress(_) => {
                            if ui.button("Cancel") {
                                return Some(NodeEvent::button_press("Cancel", *entity));
                            }

                            ui.same_line();
                            if ui.button("Pause") {
                                return Some(NodeEvent::button_press("Pause", *entity));
                            }
                        }
                        EventStatus::Completed(_) => {
                            if ui.button("Reset") {
                                return Some(NodeEvent::button_press("Reset", *entity));
                            }
                        }
                        EventStatus::Paused(_) => {
                            if ui.button("Resume") {
                                return Some(NodeEvent::button_press("Resume", *entity));
                            }

                            ui.same_line();
                            if ui.button("Cancel") {
                                return Some(NodeEvent::button_press("Cancel", *entity));
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                frame_padding.end();
            }
        }

        None
    }

    fn on_event(&mut self, nodes: &mut Nodes, node_event: NodeEvent) {
        let broker = nodes.events().plugins().features().broker();

        let node_command = match node_event {
            NodeEvent::ButtonPress {
                name: "Start",
                entity,
            } => NodeCommand::Activate(entity),
            NodeEvent::ButtonPress {
                name: "Pause",
                entity,
            } => NodeCommand::Pause(entity),
            NodeEvent::ButtonPress {
                name: "Resume",
                entity,
            } => NodeCommand::Resume(entity),
            NodeEvent::ButtonPress {
                name: "Cancel",
                entity,
            } => NodeCommand::Cancel(entity),
            NodeEvent::ButtonPress {
                name: "Reset",
                entity,
            } => NodeCommand::Reset(entity),
            _ => {
                panic!("Unrecognized command")
            }
        };

        broker.try_send_node_command(node_command, None).ok();
    }
}

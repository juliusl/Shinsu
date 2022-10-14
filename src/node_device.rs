use imgui::Ui;
use imnodes::NodeScope;
use lifec::ThunkContext;

use crate::NodeContext;

mod single_io;
pub use single_io::SingleIO;

/// Implement trait to control node display and interaction,
/// 
pub trait NodeDevice {
    /// Renders the node,
    /// 
    fn render (
        &self, 
        scope: NodeScope, 
        node_context: &NodeContext, 
        thunk_context: &ThunkContext, 
        ui: &Ui 
    );
}

impl<F> NodeDevice for F
where
    F: Fn(NodeScope, &NodeContext, &ThunkContext, &Ui),
{
    fn render(
        &self,
        scope: NodeScope,
        node_context: &NodeContext,
        thunk_context: &ThunkContext,
        ui: &Ui,
    ) {
        (self)(scope, node_context, thunk_context, ui)
    }
}

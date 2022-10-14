use imgui::Ui;
use imnodes::{NodeScope, IdentifierGenerator};
use lifec::{ThunkContext, Sequence, World};

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

    fn create (
        world: &World, 
        sequence: &Sequence,
        idgen: &mut IdentifierGenerator,
    ) -> NodeContext;
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

    fn create (
        world: &World, 
        sequence: &Sequence,
        idgen: &mut IdentifierGenerator,
    ) -> NodeContext {
        SingleIO::create(world, sequence, idgen)
    }
}

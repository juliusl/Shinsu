use imnodes::{Link, LinkId};
use lifec::prelude::{Connection, Component, DenseVecStorage};

/// Struct for a connection and all link related state,
///
#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct LinkContext(
    pub Connection, 
    pub Option<Link>, 
    pub Option<LinkId>
);

impl LinkContext {
    pub fn link_id(&self) -> Option<LinkId> {
        self.2.clone()
    }
}
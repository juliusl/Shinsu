use imnodes::{Link, LinkId};
use lifec::prelude::{Component, DenseVecStorage};

/// Struct for a connection and all link related state,
///
#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct LinkContext {
    pub link: Link,
    pub link_id: LinkId,
}

impl LinkContext {
    pub fn link(&self) -> Link {
        self.link
    }

    pub fn link_id(&self) -> LinkId {
        self.link_id
    }
}
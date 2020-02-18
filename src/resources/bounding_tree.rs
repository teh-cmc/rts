use crate::resources::{DeltaTime, Raylib};
use collision::{dbvt::DynamicBoundingVolumeTree, prelude::*};
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BoundingTree {
    // inner: DynamicBoundingVolumeTree<_>,
}

impl BoundingTree {
    pub fn new() -> Self {
        Self {}
        // let inner = DynamicBoundingVolumeTree::new();
        // Self { inner }
    }

    pub fn update(&mut self) {}
}

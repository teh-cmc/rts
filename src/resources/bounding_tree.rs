use crate::resources::{DeltaTime, Raylib};
use collision::{
    dbvt::{DynamicBoundingVolumeTree, TreeValue},
    prelude::*,
    Aabb3,
};
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct BoundingValue {
    aabb: Aabb3<f32>,
}

impl TreeValue for BoundingValue {
    type Bound = Aabb3<f32>;

    fn bound(&self) -> &Self::Bound {
        &self.aabb
    }

    fn get_bound_with_margin(&self) -> Self::Bound {
        self.aabb.clone()
    }
}

#[derive(Debug)]
pub struct BoundingTree {
    inner: DynamicBoundingVolumeTree<BoundingValue>,
}

impl BoundingTree {
    pub fn new() -> Self {
        let inner = DynamicBoundingVolumeTree::new();
        Self { inner }
    }

    pub fn update(&mut self) {}
}

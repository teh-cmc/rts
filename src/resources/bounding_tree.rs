use crate::{
    components::Vec3D,
    resources::{DeltaTime, Raylib},
};
use cgmath::Point3;
use collision::{
    dbvt::{DynamicBoundingVolumeTree, TreeValue},
    prelude::*,
    Aabb3,
};
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct BoundingValue {
    e: Entity,
    aabb: Aabb3<f32>,
}

impl BoundingValue {
    pub fn new(e: Entity, pos: Vec3D, dim: Vec3D) -> Self {
        let pos1: (_, _, _) = pos.into();
        let pos2: (_, _, _) = (pos + dim).into();
        let aabb = Aabb3::new(pos1.into(), pos2.into());
        Self { e, aabb }
    }
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

    pub fn update_entity(&mut self, e: Entity, pos: Vec3D, dim: Vec3D) {
        self.inner.insert(BoundingValue::new(e, pos, dim));
    }

    pub fn refresh(&mut self) {
        self.inner.tick()
    }
}

use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use collision::{
    dbvt::{
        ContinuousVisitor, DiscreteVisitor, DynamicBoundingVolumeTree, FrustumVisitor, TreeValue,
    },
    prelude::*,
    Aabb3, Frustum, Ray3,
};
use specs::{prelude::*, WorldExt};
use std::{cell::UnsafeCell, collections::HashMap};

// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct BoundingValue {
    e: Entity,
    aabb: Aabb3<f32>,
}

impl BoundingValue {
    pub fn new(e: Entity, min: Point3, max: Point3) -> Self {
        let aabb = Aabb3::new(*min, *max);
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
    entity_mappings: HashMap<Entity, usize>,
}

impl BoundingTree {
    pub fn new() -> Self {
        let inner = DynamicBoundingVolumeTree::new();
        let entity_mappings = HashMap::with_capacity(8192);
        Self {
            inner,
            entity_mappings,
        }
    }
}

impl BoundingTree {
    pub fn update_entity(&mut self, e: Entity, min: Point3, max: Point3) {
        let this = UnsafeCell::new(self);
        let mutself = move || -> &mut Self { unsafe { *this.get() } };
        mutself()
            .entity_mappings
            .entry(e)
            .and_modify(|idx| {
                mutself()
                    .inner
                    .update_node(*idx, BoundingValue::new(e, min, max))
            })
            .or_insert_with(|| mutself().inner.insert(BoundingValue::new(e, min, max)));
    }

    pub fn refresh(&mut self) {
        self.inner.tick()
    }
}

impl BoundingTree {
    pub fn test_frustum(&self, frust: &Frustum<f32>) -> impl Iterator<Item = Entity> + '_ {
        let mut vis = FrustumVisitor::<_, BoundingValue>::new(frust);
        self.inner.query(&mut vis).into_iter().map(|(bv, _)| bv.e)
    }

    pub fn test_ray(&self, r: &Ray3<f32>) -> impl Iterator<Item = Entity> + '_ {
        let mut vis = ContinuousVisitor::<_, BoundingValue>::new(r);
        self.inner.query(&mut vis).into_iter().map(|(bv, _)| bv.e)
    }
}

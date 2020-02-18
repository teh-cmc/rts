use crate::{components, resources};

use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct BoundingTree;

impl<'a> System<'a> for BoundingTree {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, resources::BoundingTree>,
        ReadStorage<'a, components::Pos3D>,
        ReadStorage<'a, components::Dim3D>,
        WriteStorage<'a, components::Pos3DInvalidated>,
    );

    fn run(&mut self, (entities, mut bt, pos3Ds, dim3Ds, mut moved): Self::SystemData) {
        for (e, pos, dim, _) in (&entities, &pos3Ds, &dim3Ds, &moved).join() {
            eprintln!("{:?} invalidated", e);
        }
        for e in entities.join() {
            moved.remove(e);
        }
        bt.update();
    }
}

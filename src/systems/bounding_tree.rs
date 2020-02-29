use crate::{components::prelude::*, resources::prelude::*};
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct BoundingTree;

impl<'a> System<'a> for BoundingTree {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, ResrcBoundingTree>,
        ReadStorage<'a, CompPos3D>,
        ReadStorage<'a, CompMesh>,
        WriteStorage<'a, CompPos3DInvalidated>,
    );

    fn run(&mut self, (entities, mut bt, pos3Ds, meshes, mut moved): Self::SystemData) {
        for (e, pos, mesh, _) in (&entities, &pos3Ds, &meshes, &moved).join() {
            match mesh {
                CompMesh::Rect { .. } => {}
                CompMesh::Line { .. } => {}
                CompMesh::Cube { dimensions } => {
                    eprintln!("{:?} invalidated", e);
                    bt.update_entity(e, pos.0, *dimensions);
                }
            }
        }
        bt.refresh();

        for e in entities.join() {
            moved.remove(e);
        }
    }
}

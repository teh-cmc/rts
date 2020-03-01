use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct BoundingTree;

impl<'a> System<'a> for BoundingTree {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, ResrcBoundingTree>,
        ReadStorage<'a, CompModel3D>,
        ReadStorage<'a, CompTransform3D>,
        WriteStorage<'a, CompInvalidated>,
    );

    fn run(&mut self, (entities, mut bt, models, transforms, mut invalidated): Self::SystemData) {
        // TODO(cmc): can probably greatly simplify all of this with swizzling
        // and a better maths module.
        for (e, model, transform, _) in (&entities, &models, &transforms, &invalidated).join() {
            let bbox = model.0.meshes()[0].mesh_bounding_box();
            let min: Vec4 = (bbox.min.x, bbox.min.y, bbox.min.z, 1.).into();
            let max: Vec4 = (bbox.max.x, bbox.max.y, bbox.max.z, 1.).into();
            let min = *transform.0 * *min;
            let max = *transform.0 * *max;

            eprintln!("{:?} invalidated", e);
            bt.update_entity(
                e,
                (min.x, min.y, min.z).into(),
                (max.x, max.y, max.z).into(),
            );
        }

        bt.refresh();
        for e in entities.join() {
            invalidated.remove(e);
        }
    }
}

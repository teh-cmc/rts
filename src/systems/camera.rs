use crate::resources::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Camera;

impl<'a> System<'a> for Camera {
    type SystemData = (
        ReadExpect<'a, ResrcRaylib>,
        ReadExpect<'a, ResrcDeltaTime>,
        WriteExpect<'a, ResrcCamera>,
    );

    fn run(&mut self, (rl, delta, mut cam): Self::SystemData) {
        cam.update(&rl, &delta);
    }
}

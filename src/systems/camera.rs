use crate::resources;

use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Camera;

impl<'a> System<'a> for Camera {
    type SystemData = (
        ReadExpect<'a, resources::Raylib>,
        ReadExpect<'a, resources::DeltaTime>,
        WriteExpect<'a, resources::Camera>,
    );

    fn run(&mut self, (rl, delta, mut cam): Self::SystemData) {
        cam.update(&rl, &delta);
    }
}

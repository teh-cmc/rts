use crate::resources;

use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Mouse;

impl<'a> System<'a> for Mouse {
    type SystemData = (
        ReadExpect<'a, resources::Raylib>,
        WriteExpect<'a, resources::MouseState>,
    );

    fn run(&mut self, (rl, mut mouse): Self::SystemData) {
        mouse.update(&rl);
    }
}

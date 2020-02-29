use crate::resources::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Mouse;

impl<'a> System<'a> for Mouse {
    type SystemData = (
        ReadExpect<'a, ResrcRaylib>,
        WriteExpect<'a, ResrcMouseState>,
    );

    fn run(&mut self, (rl, mut mouse): Self::SystemData) {
        mouse.update(&rl);
    }
}

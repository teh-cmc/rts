use crate::resources;

use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct BoundingTree;

impl<'a> System<'a> for BoundingTree {
    type SystemData = (
        ReadExpect<'a, resources::Raylib>,
        ReadExpect<'a, resources::DeltaTime>,
        WriteExpect<'a, resources::BoundingTree>,
    );

    fn run(&mut self, (rl, delta, mut bt): Self::SystemData) {
        bt.update();
    }
}

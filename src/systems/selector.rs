use crate::{components, resources};
use cgmath::prelude::*;
use components::Vec2D;
use raylib::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

enum SelectorState {
    Idle,
    Selecting(Entity, Vec2D),
    Confirmed(Entity),
}

pub struct Selector {
    state: SelectorState,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            state: SelectorState::Idle,
        }
    }
}

impl<'a> System<'a> for Selector {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, resources::Camera>,
        ReadExpect<'a, resources::MouseState>,
        ReadStorage<'a, components::Pos3D>,
        WriteStorage<'a, components::Pos2D>,
        WriteStorage<'a, components::Dim2D>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (entities, cam, mouse, pos3Ds, mut pos2Ds, mut dim2Ds) = sys_data;
        match self.state {
            SelectorState::Idle => {
                if mouse.is_pressed(0) {
                    let pos = mouse.position();
                    let e = entities
                        .build_entity()
                        .with(pos.into(), &mut pos2Ds)
                        .build();
                    self.state = SelectorState::Selecting(e, pos);
                }
            }
            SelectorState::Selecting(e, pos2) => {
                let pos1 = mouse.position();
                let dim = pos1 - pos2;
                dim2Ds.insert(e, dim.into()).unwrap();
                if mouse.is_released(0) {
                    self.state = SelectorState::Confirmed(e);
                }
            }
            SelectorState::Confirmed(e) => {
                entities.delete(e).unwrap();
                self.state = SelectorState::Idle;

                // for components::Pos3D(pos) in pos3Ds.join() {}
            }
        }
    }
}

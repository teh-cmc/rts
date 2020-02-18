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
            SelectorState::Selecting(e, mut pos1) => {
                let mut pos2 = mouse.position();
                let mut dim = pos2 - pos1;
                if dim.x.is_negative() {
                    dim.x = dim.x.abs();
                    std::mem::swap(&mut pos1.x, &mut pos2.x);
                }
                if dim.y.is_negative() {
                    dim.y = dim.y.abs();
                    std::mem::swap(&mut pos1.y, &mut pos2.y);
                }

                *pos2Ds.get_mut(e).unwrap() = pos1.into();
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

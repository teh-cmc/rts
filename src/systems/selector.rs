use crate::{components, resources};

use specs::prelude::*;

// -----------------------------------------------------------------------------

enum SelectorState {
    Idle,
    Selecting(Entity, components::Pos2D),
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
        let (entities, _cam, mouse, _positions, mut pos2Ds, mut dim2Ds) = sys_data;
        match self.state {
            SelectorState::Idle => {
                if mouse.is_pressed(0) {
                    let pos = mouse.position();
                    let e = entities.build_entity().with(pos, &mut pos2Ds).build();
                    self.state = SelectorState::Selecting(e, pos);
                }
            }
            SelectorState::Selecting(e, _upper_left @ components::Pos2D(x1, y1)) => {
                let pos = mouse.position();
                let dim = components::Dim2D(pos.0 - x1, pos.1 - y1);
                dim2Ds.insert(e, dim).unwrap();
                if mouse.is_released(0) {
                    self.state = SelectorState::Confirmed(e);
                }
            }
            SelectorState::Confirmed(e) => {
                entities.delete(e).unwrap();
                self.state = SelectorState::Idle;
            }
        }
    }
}

use crate::{components, resources::Raylib};
use components::Vec2D;

// -----------------------------------------------------------------------------

// TODO(cmc): bitsets
// TODO(cmc): portable design (i.e. abstract raylib)
pub struct MouseState {
    pos: Vec2D,
    pressed: [bool; 8],
    released: [bool; 8],
    down: [bool; 8],
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            pos: (0, 0).into(),
            pressed: Default::default(),
            released: Default::default(),
            down: Default::default(),
        }
    }
}

impl MouseState {
    pub fn position(&self) -> Vec2D {
        self.pos
    }

    pub fn is_pressed(&self, button: usize) -> bool {
        self.pressed[button]
    }

    pub fn is_released(&self, button: usize) -> bool {
        self.released[button]
    }

    pub fn is_down(&self, button: usize) -> bool {
        self.down[button]
    }

    pub fn update(&mut self, rl: &Raylib) {
        use raylib::prelude::*;
        use MouseButton::*;
        rl.read(|rl| {
            self.pos = (rl.get_mouse_x(), rl.get_mouse_y()).into();

            self.pressed[0] = rl.is_mouse_button_pressed(MOUSE_LEFT_BUTTON);
            self.pressed[1] = rl.is_mouse_button_pressed(MOUSE_RIGHT_BUTTON);
            self.pressed[2] = rl.is_mouse_button_pressed(MOUSE_MIDDLE_BUTTON);

            self.released[0] = rl.is_mouse_button_released(MOUSE_LEFT_BUTTON);
            self.released[1] = rl.is_mouse_button_released(MOUSE_RIGHT_BUTTON);
            self.released[2] = rl.is_mouse_button_released(MOUSE_MIDDLE_BUTTON);

            self.down[0] = rl.is_mouse_button_down(MOUSE_LEFT_BUTTON);
            self.down[1] = rl.is_mouse_button_down(MOUSE_RIGHT_BUTTON);
            self.down[2] = rl.is_mouse_button_down(MOUSE_MIDDLE_BUTTON);
        });
    }
}

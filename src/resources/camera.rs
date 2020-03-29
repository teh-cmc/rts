use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use raylib::prelude::*;
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Camera {
    inner: Camera3D,

    pos: Vec3,
    x_rad: f32,
    y_rad: f32,
    radius: f32,
}

impl Camera {
    const PI: f32 = consts::PI as f32;
    const Y: f32 = 30.0;

    pub fn new(inner: Camera3D) -> Self {
        Self {
            pos: inner.position.to_array().into(),
            inner,
            x_rad: Self::PI * 0.25,
            y_rad: -Self::PI * 0.25,
            radius: 1000.0,
        }
    }

    pub fn raw(&self) -> Camera3D {
        self.inner
    }

    // TODO(cmc): should be bitsets obviously
    // https://gamedev.stackexchange.com/a/159314
    pub fn update(&mut self, rl: &ResrcRaylib, delta: &ResrcDeltaTime) {
        // TODO(cmc): kbd state
        let ((l, u, r, d), zoom) = rl.read(|rl| {
            let dir = (
                rl.is_key_down(KeyboardKey::KEY_A),
                rl.is_key_down(KeyboardKey::KEY_W),
                rl.is_key_down(KeyboardKey::KEY_D),
                rl.is_key_down(KeyboardKey::KEY_S),
            );
            let zoom = rl.get_mouse_wheel_move();

            (dir, zoom)
        });

        if l {
            self.pos.x += delta.0 * self.x_rad.cos();
            self.pos.z -= delta.0 * self.x_rad.sin();
        }
        if r {
            self.pos.x -= delta.0 * self.x_rad.cos();
            self.pos.z += delta.0 * self.x_rad.sin();
        }
        if u {
            self.pos.x += delta.0 * self.x_rad.sin();
            self.pos.z += delta.0 * self.x_rad.cos();
        }
        if d {
            self.pos.x -= delta.0 * self.x_rad.sin();
            self.pos.z -= delta.0 * self.x_rad.cos();
        }

        self.y_rad += zoom as f32 * delta.0 * 10. * Self::PI / 180.0;
        // self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(-Self::PI * 0.15);
        self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(Self::PI / 3.);
        self.pos.y = Self::Y * 2. * self.y_rad.abs();

        self.inner.position = (self.pos.x, self.pos.y, self.pos.z).into();
        self.inner.target = (
            self.pos.x + self.radius * self.x_rad.sin() * self.y_rad.cos(),
            self.pos.y + self.radius * self.y_rad.sin(),
            self.pos.z + self.radius * self.x_rad.cos() * self.y_rad.cos(),
        )
            .into();
    }
}

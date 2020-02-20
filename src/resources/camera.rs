use crate::{
    components,
    resources::{DeltaTime, Raylib},
};
use raylib::prelude::*;
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Camera {
    inner: Camera3D,

    x_rad: f32,
    y_rad: f32,
    radius: f32,
}

impl Camera {
    const PI: f32 = consts::PI as f32;
    const Y: f32 = 30.0;

    pub fn new(inner: Camera3D) -> Self {
        Self {
            inner,
            x_rad: Self::PI * 0.25,
            y_rad: -Self::PI * 0.25,
            radius: 100.0,
        }
    }

    pub fn raw(&self) -> Camera3D {
        self.inner
    }

    pub fn set_pos(&mut self, rl: &Raylib, pos: components::Pos3D) -> components::Pos3D {
        let pos_old = components::Vec3D::from((
            self.inner.position.x,
            self.inner.position.y,
            self.inner.position.z,
        ))
        .into();
        self.inner.position = pos.into();
        self.inner.target = Vector3::new(
            self.inner.position.x + self.radius * self.x_rad.sin() * self.y_rad.cos(),
            self.inner.position.y + self.radius * self.y_rad.sin(),
            self.inner.position.z + self.radius * self.x_rad.cos() * self.y_rad.cos(),
        );
        pos_old
    }

    // https://gamedev.stackexchange.com/a/159314
    // TODO(cmc): should be bitsets obviously
    pub fn update(&mut self, rl: &Raylib, delta: &DeltaTime) {
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
            self.inner.position.x += delta.0 * self.x_rad.cos();
            self.inner.position.z -= delta.0 * self.x_rad.sin();
        }
        if r {
            self.inner.position.x -= delta.0 * self.x_rad.cos();
            self.inner.position.z += delta.0 * self.x_rad.sin();
        }
        if u {
            self.inner.position.x += delta.0 * self.x_rad.sin();
            self.inner.position.z += delta.0 * self.x_rad.cos();
        }
        if d {
            self.inner.position.x -= delta.0 * self.x_rad.sin();
            self.inner.position.z -= delta.0 * self.x_rad.cos();
        }

        self.y_rad += zoom as f32 * delta.0 * 10. * Self::PI / 180.0;
        self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(-Self::PI * 0.15);
        self.inner.position.y = Self::Y * 2. * self.y_rad.abs();

        self.inner.target = Vector3::new(
            self.inner.position.x + self.radius * self.x_rad.sin() * self.y_rad.cos(),
            self.inner.position.y + self.radius * self.y_rad.sin(),
            self.inner.position.z + self.radius * self.x_rad.cos() * self.y_rad.cos(),
        );
    }
}

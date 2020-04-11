use crate::{components::prelude::*, maths::prelude::*, resources::prelude::*};
use raylib::prelude::*;
use specs::{prelude::*, WorldExt};

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    RTS,
    FREE,
    ORBITAL,
}

pub struct Camera {
    inner: Camera3D,
    mode: Mode,

    updates_prev: Option<Updates>,
    updater: Box<dyn Updater + Send + Sync + 'static>,
}

impl Camera {
    pub fn new(inner: Camera3D, mode: Mode) -> Self {
        let updater = Box::new(updaters::RTS::default());
        Self {
            inner,
            mode,
            updates_prev: None,
            updater,
        }
    }

    pub fn raw(&self) -> Camera3D {
        self.inner
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}

impl Camera {
    pub fn update(&mut self, rl: &ResrcRaylib, delta: &ResrcDeltaTime) {
        // TODO(cmc): kbd state
        rl.read(|rl| {
            if rl.is_key_released(KeyboardKey::KEY_F1) {
                self.mode = Mode::RTS;
                self.updater = Box::new(updaters::RTS::default());
            }
            if rl.is_key_released(KeyboardKey::KEY_F2) {
                self.mode = Mode::FREE;
                self.updater = Box::new(updaters::Free::default());
            }
            if rl.is_key_released(KeyboardKey::KEY_F3) {
                self.mode = Mode::ORBITAL;
                self.updater = Box::new(updaters::Orbital::default());
            }
        });

        let updates = if rl.read(|rl| rl.is_mouse_button_up(MouseButton::MOUSE_LEFT_BUTTON)) {
            Updates::from_input(rl, None)
        } else {
            Updates::from_input(rl, self.updates_prev.as_ref())
        };

        let (pos, target) = self.updater.update(delta, &updates);
        self.updates_prev = updates.into();

        self.inner.position = (pos.x, pos.y, pos.z).into();
        self.inner.target = (target.x, target.y, target.z).into();
    }
}

// -----------------------------------------------------------------------------

trait Updater {
    fn update(&mut self, delta: &ResrcDeltaTime, updates: &Updates) -> (Vec3, Point3);
}

#[derive(Debug, Clone)]
struct Updates {
    mov_left: bool,
    mov_up: bool,
    mov_right: bool,
    mov_down: bool,

    rot_left: bool,
    rot_up: bool,
    rot_right: bool,
    rot_down: bool,

    mouse_pos: Vec2,
    mouse_diff: Vec2,

    zoom: i32,
}

impl Updates {
    // TODO(cmc): kbd state + no rl ref
    pub fn from_input(rl: &ResrcRaylib, prev: Option<&Self>) -> Self {
        let ((mov_left, mov_up, mov_right, mov_down), zoom) = rl.read(|rl| {
            let dir = (
                rl.is_key_down(KeyboardKey::KEY_A),
                rl.is_key_down(KeyboardKey::KEY_W),
                rl.is_key_down(KeyboardKey::KEY_D),
                rl.is_key_down(KeyboardKey::KEY_S),
            );
            let zoom = rl.get_mouse_wheel_move();

            (dir, zoom)
        });

        let mouse_pos = rl.read(|rl| rl.get_mouse_position());
        let mouse_pos: Vec2 = (mouse_pos.x, mouse_pos.y).into();
        let mouse_diff = prev
            .map_or_else(|| (0., 0.).into(), |u| *mouse_pos - *u.mouse_pos)
            .into();

        Self {
            mov_left,
            mov_up,
            mov_right,
            mov_down,

            rot_left: false,
            rot_up: false,
            rot_right: false,
            rot_down: false,

            mouse_pos,
            mouse_diff,

            zoom,
        }
    }
}

mod updaters {
    use super::*;

    // -----------------------------------------------------------------------------

    #[derive(Debug, Clone)]
    pub struct RTS {
        pos: Vec3,
        x_rad: f32,
        y_rad: f32,
        radius: f32,
        speed_multiplier: f32,
    }

    impl RTS {
        const PI: f32 = consts::PI as f32;
        const Y: f32 = 30.0;
    }

    impl Default for RTS {
        fn default() -> Self {
            Self {
                pos: (0., 0., 0.).into(),
                x_rad: Self::PI * 0.25,
                y_rad: -Self::PI * 0.25,
                radius: 1000.0,
                speed_multiplier: 50.,
            }
        }
    }

    impl Updater for RTS {
        fn update(&mut self, delta: &ResrcDeltaTime, updates: &Updates) -> (Vec3, Point3) {
            let delta = delta.0 * self.speed_multiplier;
            if updates.mov_left {
                self.pos.x += delta * self.x_rad.cos();
                self.pos.z -= delta * self.x_rad.sin();
            }
            if updates.mov_right {
                self.pos.x -= delta * self.x_rad.cos();
                self.pos.z += delta * self.x_rad.sin();
            }
            if updates.mov_up {
                self.pos.x += delta * self.x_rad.sin();
                self.pos.z += delta * self.x_rad.cos();
            }
            if updates.mov_down {
                self.pos.x -= delta * self.x_rad.sin();
                self.pos.z -= delta * self.x_rad.cos();
            }

            self.y_rad += updates.zoom as f32 * delta * 10. * Self::PI / 180.0;
            // self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(-Self::PI * 0.15);
            self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(Self::PI / 3.);
            self.pos.y = Self::Y * 2. * self.y_rad.abs();

            // https://gamedev.stackexchange.com/a/159314
            let target = (
                self.pos.x + self.radius * self.x_rad.sin() * self.y_rad.cos(),
                self.pos.y + self.radius * self.y_rad.sin(),
                self.pos.z + self.radius * self.x_rad.cos() * self.y_rad.cos(),
            );

            (self.pos.clone(), target.into())
        }
    }

    // -----------------------------------------------------------------------------

    #[derive(Debug, Clone)]
    pub struct Orbital {
        elapsed: f32,
        radius: f32,
        speed_multiplier: f32,
    }

    impl Default for Orbital {
        fn default() -> Self {
            Self {
                elapsed: 0.,
                radius: 100.0,
                speed_multiplier: 1.5,
            }
        }
    }

    impl Updater for Orbital {
        fn update(&mut self, delta: &ResrcDeltaTime, updates: &Updates) -> (Vec3, Point3) {
            self.radius -= updates.zoom as f32 * 100.0;
            self.elapsed += delta.0 * self.speed_multiplier;

            let pos = (
                self.elapsed.sin() * self.radius,
                100.,
                self.elapsed.cos() * self.radius,
            );
            let target = (0., 0., 0.);

            (pos.into(), target.into())
        }
    }

    // -----------------------------------------------------------------------------

    #[derive(Debug, Clone)]
    pub struct Free {
        pos: Vec3,
        dir: Vec3,
        target: Point3,
        speed_multiplier: f32,
        pitch: f32,
        yaw: f32,
        sensitivity: f32,
    }

    impl Free {
        const MAX_PITCH: f32 = Self::PI / 4.01;
        const PI: f32 = consts::PI as f32;
    }

    impl Default for Free {
        fn default() -> Self {
            Self {
                pos: (0., 0., 0.).into(),
                dir: (0., 0., -1.).into(),
                target: (0., 0., 0.).into(),
                speed_multiplier: 50.,
                pitch: 0.,
                yaw: -Self::MAX_PITCH,
                sensitivity: 0.01,
            }
        }
    }

    impl Updater for Free {
        fn update(&mut self, delta: &ResrcDeltaTime, updates: &Updates) -> (Vec3, Point3) {
            let delta = delta.0 * self.speed_multiplier;

            self.pitch += updates.mouse_diff.y * self.sensitivity;
            self.pitch = self.pitch.max(-Self::MAX_PITCH).min(Self::MAX_PITCH);
            self.yaw -= updates.mouse_diff.x * self.sensitivity;
            self.dir = {
                let mut dir = self.dir.normalize();
                dir.x = self.yaw.cos() + self.pitch.cos();
                dir.y = self.pitch.sin();
                dir.z = self.yaw.sin() + self.pitch.cos();
                dir.normalize().into()
            };

            let up = (0., 1., 0.).into();
            let left = self.dir.cross(up).normalize();

            if updates.mov_left {
                *self.pos -= delta * left;
            }
            if updates.mov_right {
                *self.pos += delta * left;
            }
            if updates.mov_up {
                *self.pos += delta * *self.dir;
            }
            if updates.mov_down {
                *self.pos -= delta * *self.dir;
            }

            let target = *self.pos + *self.dir;

            (self.pos.clone(), (target.x, target.y, target.z).into())
        }
    }
}

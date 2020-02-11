use raylib::prelude::*;
use specs::{prelude::*, storage::HashMapStorage, WorldExt};

// -----------------------------------------------------------------------------

pub mod resources {
    #[derive(Default)]
    pub struct DeltaTime(f32);
}

pub mod components {
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};

    #[derive(Clone, Debug)]
    pub struct Pos(pub Vector3);
    impl Component for Pos {
        type Storage = VecStorage<Self>;
    }

    #[derive(Default, Debug)]
    pub struct Selected;
    impl Component for Selected {
        type Storage = NullStorage<Self>;
    }
}

pub mod systems {
    use super::components;
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};
    use std::{
        ops::{Deref, DerefMut},
        sync::{Arc, Mutex},
    };

    pub struct Renderer {
        // rl_handle: Arc<Mutex<RaylibHandle>>,
    // gfx_handle: RaylibThread,
    }

    // impl Renderer {
    //     pub fn new(rl_handle: Arc<Mutex<RaylibHandle>>, gfx_handle: RaylibThread)
    // -> Self {         Self {
    //             rl_handle,
    //             gfx_handle,
    //         }
    //     }
    // }

    impl<'a> System<'a> for Renderer {
        type SystemData = (
            WriteExpect<'a, super::RawDrawHandle>,
            ReadExpect<'a, super::Camera>,
            ReadStorage<'a, components::Pos>,
        );

        fn run(&mut self, (mut d, cam, positions): Self::SystemData) {
            let d = d.lol();
            d.clear_background(Color::DARKGRAY);
            {
                let mut d2 = d.begin_mode_3D(cam.deref().inner);
                for pos in positions.join() {
                    d2.draw_cube(pos.0, 2.0, 2.0, 2.0, Color::RED);
                    d2.draw_cube_wires(pos.0, 2.0, 2.0, 2.0, Color::BLACK);
                }
            }

            d.draw_rectangle(10, 10, 220, 70, Color::SKYBLUE);
            d.draw_rectangle_lines(10, 10, 220, 70, Color::BLUE);
            d.draw_text(
                "First person camera default controls:",
                20,
                20,
                10,
                Color::BLACK,
            );
            d.draw_text("- Move with keys: W, A, S, D", 40, 40, 10, Color::DARKGRAY);
            d.draw_text("- Mouse move to look around", 40, 60, 10, Color::DARKGRAY);
        }
    }
}

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

    pub fn new(inner: Camera3D) -> Self {
        Self {
            inner,
            x_rad: Self::PI * 0.25,
            y_rad: -Self::PI * 0.25,
            radius: 100.0,
        }
    }

    // https://gamedev.stackexchange.com/a/159314
    pub fn update(&mut self, delta: f32, (l, u, r, d): (bool, bool, bool, bool)) {
        if l {
            self.inner.position.x += delta * self.x_rad.cos();
            self.inner.position.z -= delta * self.x_rad.sin();
        }
        if r {
            self.inner.position.x -= delta * self.x_rad.cos();
            self.inner.position.z += delta * self.x_rad.sin();
        }
        if u {
            self.inner.position.x += delta * self.x_rad.sin();
            self.inner.position.z += delta * self.x_rad.cos();
        }
        if d {
            self.inner.position.x -= delta * self.x_rad.sin();
            self.inner.position.z -= delta * self.x_rad.cos();
        }

        self.inner.target = Vector3::new(
            self.inner.position.x + self.radius * self.x_rad.sin() * self.y_rad.cos(),
            self.inner.position.y + self.radius * self.y_rad.sin(),
            self.inner.position.z + self.radius * self.x_rad.cos() * self.y_rad.cos(),
        );
    }
}

// -----------------------------------------------------------------------------

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

use std::sync::{Arc, RwLock};

pub struct RawDrawHandle(*mut ());
unsafe impl Send for RawDrawHandle {}
unsafe impl Sync for RawDrawHandle {}

impl RawDrawHandle {
    pub fn lol(&mut self) -> &mut RaylibDrawHandle {
        unsafe { std::mem::transmute(&mut self.0) }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("RTS")
        .build();

    let mut cam = {
        let inner = Camera3D::perspective(
            (0.0, 30.0, 0.0).into(),
            Vector3::zero(),
            (0.0, 1.0, 0.0).into(),
            60.0,
        );

        rl.set_camera_mode(&inner, CameraMode::CAMERA_CUSTOM);
        rl.set_target_fps(60);

        let mut cam = Camera::new(inner);
        cam.update(0.0, (false, false, false, false));

        cam
    };

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with_thread_local(systems::Renderer {})
        .build();
    dispatcher.setup(&mut world);

    world
        .create_entity()
        .with(components::Pos(Vector3::zero()))
        .build();
    world
        .create_entity()
        .with(components::Pos((-10.0, 0.0, 0.0).into()))
        .build();

    while !rl.window_should_close() {
        let delta = rl.get_frame_time() * 10.0;

        cam.update(
            delta,
            (
                rl.is_key_down(KeyboardKey::KEY_A),
                rl.is_key_down(KeyboardKey::KEY_W),
                rl.is_key_down(KeyboardKey::KEY_D),
                rl.is_key_down(KeyboardKey::KEY_S),
            ),
        );

        world.insert(cam.clone());
        let mut d = rl.begin_drawing(&thread);
        world.insert(RawDrawHandle(unsafe { std::mem::transmute(&mut d) }));

        dispatcher.dispatch(&mut world);
        world.maintain();
    }
}

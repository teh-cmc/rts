#![feature(bindings_after_at, type_name_of_val)]

use raylib::prelude::*;
use specs::{prelude::*, storage::HashMapStorage, WorldExt};

// -----------------------------------------------------------------------------

// TODO(cmc): never reference raylib in there.
pub mod resources {
    use super::components;
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};
    use std::sync::{Arc, RwLock};

    // TODO(cmc): shouldn't have a lock, ECS handles sync.
    pub struct Raylib(Arc<RwLock<RaylibHandle>>);
    impl Raylib {
        pub fn new(rl: RaylibHandle) -> Self {
            Self(Arc::new(RwLock::new(rl)))
        }

        pub fn read<T>(&self, mut f: impl FnMut(&RaylibHandle) -> T) -> T {
            f(&*self.0.try_read().unwrap())
        }

        pub fn write<T>(&mut self, mut f: impl FnMut(&mut RaylibHandle) -> T) -> T {
            f(&mut *self.0.try_write().unwrap())
        }

        pub fn draw(&mut self, tl: &RaylibThread, mut f: impl FnMut(&mut RaylibDrawHandle)) {
            let mut guard = self.0.try_write().unwrap();
            let mut d = guard.begin_drawing(tl);
            f(&mut d)
        }
    }

    impl Clone for Raylib {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }

    // -----------------------------------------------------------------------------

    #[derive(Default)]
    pub struct DeltaTime(pub f32);

    // -----------------------------------------------------------------------------

    // TODO(cmc): bitsets
    // TODO(cmc): portable design (i.e. abstract raylib)
    #[derive(Default)]
    pub struct MouseState {
        pos: (i32, i32),

        pressed: [bool; 8],
        released: [bool; 8],
        down: [bool; 8],
    }

    impl MouseState {
        pub fn position(&self) -> components::Pos2D {
            self.pos.into()
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
            use MouseButton::*;
            rl.read(|rl| {
                self.pos = (rl.get_mouse_x(), rl.get_mouse_y());

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
}

// TODO(cmc): never reference raylib in there.
pub mod components {
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};

    #[derive(Clone, Debug)]
    pub struct Pos3D(pub Vector3);
    impl Component for Pos3D {
        type Storage = VecStorage<Self>;
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Pos2D(pub i32, pub i32);
    impl Component for Pos2D {
        type Storage = HashMapStorage<Self>;
    }
    impl From<(i32, i32)> for Pos2D {
        fn from((x, y): (i32, i32)) -> Self {
            Self(x, y)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Dim2D(pub i32, pub i32);
    impl Component for Dim2D {
        type Storage = HashMapStorage<Self>;
    }
    impl From<(i32, i32)> for Dim2D {
        fn from((x, y): (i32, i32)) -> Self {
            Self(x, y)
        }
    }

    #[derive(Default, Debug)]
    pub struct Selected;
    impl Component for Selected {
        type Storage = NullStorage<Self>;
    }
}

// TODO(cmc): never reference raylib in there.
pub mod systems {
    use super::{components, resources};
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};
    use std::{
        ops::{Deref, DerefMut},
        sync::{Arc, Mutex},
    };

    // -----------------------------------------------------------------------------

    #[derive(Default)]
    pub struct Mouse;

    impl<'a> System<'a> for Mouse {
        type SystemData = (
            ReadExpect<'a, resources::Raylib>,
            WriteExpect<'a, resources::MouseState>,
        );

        fn run(&mut self, (rl, mut mouse): Self::SystemData) {
            mouse.update(&rl);
        }
    }

    // -----------------------------------------------------------------------------

    #[derive(Default)]
    pub struct Camera;

    impl<'a> System<'a> for Camera {
        type SystemData = (
            ReadExpect<'a, resources::Raylib>,
            ReadExpect<'a, resources::DeltaTime>,
            WriteExpect<'a, resources::Camera>,
        );

        fn run(&mut self, (rl, delta, mut cam): Self::SystemData) {
            cam.update(&rl, &delta);
        }
    }

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
            let (entities, cam, mouse, positions, mut pos2Ds, mut dim2Ds) = sys_data;
            match self.state {
                SelectorState::Idle => {
                    if mouse.is_pressed(0) {
                        let pos = mouse.position();
                        let e = entities.build_entity().with(pos, &mut pos2Ds).build();
                        self.state = SelectorState::Selecting(e, pos);
                    }
                }
                SelectorState::Selecting(e, upper_left @ components::Pos2D(x1, y1)) => {
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

    // -----------------------------------------------------------------------------

    #[derive(Default)]
    pub struct Renderer;

    impl<'a> System<'a> for Renderer {
        type SystemData = (
            WriteExpect<'a, resources::Raylib>,
            ReadExpect<'a, resources::Camera>,
            ReadStorage<'a, components::Pos3D>,
            ReadStorage<'a, components::Pos2D>,
            ReadStorage<'a, components::Dim2D>,
        );

        fn run(&mut self, (mut rl, cam, pos3Ds, pos2Ds, dim2Ds): Self::SystemData) {
            // TODO(cmc): safety note.
            let thread: RaylibThread = unsafe { std::mem::transmute(()) };
            let (swidth, sheight) = (rl.read(|rl| rl.get_screen_width() - 100), 40);
            rl.draw(&thread, |d| {
                d.clear_background(Color::DARKGRAY);

                {
                    let mut d2 = d.begin_mode_3D(cam.raw());
                    for pos in pos3Ds.join() {
                        d2.draw_cube(pos.0, 2.0, 2.0, 2.0, Color::RED);
                        d2.draw_cube_wires(pos.0, 2.0, 2.0, 2.0, Color::BLACK);
                    }
                }

                use components::{Dim2D, Pos2D};
                for (&Pos2D(x, y), &Dim2D(w, h)) in (&pos2Ds, &dim2Ds).join() {
                    d.draw_rectangle(x, y, w, h, Color::GREEN.fade(0.1));
                    d.draw_rectangle_lines(x, y, w, h, Color::GREEN);
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

                d.draw_fps(swidth, sheight);
            });
        }
    }
}

// -----------------------------------------------------------------------------

fn main() {
    const WINDOW_WIDTH: i32 = 1280;
    const WINDOW_HEIGHT: i32 = 720;
    let (rl, _) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("RTS")
        .build();

    let mut world = World::new();
    let mut dispatcher = {
        // TODO(cmc): macro this?
        use std::any::type_name_of_val as sys_id;
        let mouse = systems::Mouse::default();
        let mouse_id = sys_id(&mouse);
        let cam = systems::Camera::default();
        let cam_id = sys_id(&cam);
        let selector = systems::Selector::default();
        let selector_id = sys_id(&selector);
        let renderer = systems::Renderer::default();

        DispatcherBuilder::new()
            .with(mouse, mouse_id, &[])
            .with(cam, cam_id, &[])
            .with(selector, selector_id, &[mouse_id])
            .with_thread_local(renderer)
            .build()
    };
    dispatcher.setup(&mut world);

    let mut rl = resources::Raylib::new(rl);
    world.insert(rl.clone());

    world.insert(resources::DeltaTime(0.0));
    world.insert(resources::MouseState::default());

    let cam = {
        let inner = Camera3D::perspective(
            Vector3::zero(),
            Vector3::zero(),
            (0.0, 1.0, 0.0).into(),
            60.0,
        );

        rl.write(|rl| {
            rl.set_camera_mode(&inner, CameraMode::CAMERA_CUSTOM);
            rl.set_target_fps(120);
        });

        resources::Camera::new(inner)
    };
    world.insert(cam);

    for x in -10..=10 {
        for z in -10..=10 {
            let pos: Vector3 = (x as f32 * 4.0, 0.0, z as f32 * 4.0).into();
            world.create_entity().with(components::Pos3D(pos)).build();
        }
    }

    while !rl.read(|rl| rl.window_should_close()) {
        let delta = rl.read(|rl| rl.get_frame_time() * 50.0);
        world.write_resource::<resources::DeltaTime>().0 = delta;

        dispatcher.dispatch(&mut world);
        world.maintain();
    }
}

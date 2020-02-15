#![feature(bindings_after_at)]

use raylib::prelude::*;
use specs::{prelude::*, storage::HashMapStorage, WorldExt};

// -----------------------------------------------------------------------------

// TODO(cmc): never reference raylib in there.
pub mod resources {
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};
    use std::sync::{Arc, RwLock};

    // TODO(cmc): shouldn't have a lock, ECS handles sync.
    pub struct Raylib(pub Arc<RwLock<RaylibHandle>>);
    impl Raylib {
        pub fn new(rl: RaylibHandle) -> Self {
            Self(Arc::new(RwLock::new(rl)))
        }

        pub fn read<T>(&self, mut f: impl FnMut(&RaylibHandle) -> T) -> T {
            f(&*self.0.read().unwrap())
        }

        pub fn write<T>(&mut self, mut f: impl FnMut(&mut RaylibHandle) -> T) -> T {
            f(&mut *self.0.write().unwrap())
        }

        pub fn draw(&mut self, tl: &RaylibThread, mut f: impl FnMut(&mut RaylibDrawHandle)) {
            let mut guard = self.0.write().unwrap();
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
    pub struct DeltaTime(f32);

    // -----------------------------------------------------------------------------

    // TODO(cmc): bitsets
    // TODO(cmc): portable design (i.e. abstract raylib)
    #[derive(Default)]
    pub struct MouseState {
        pub pos: (i32, i32),

        pub pressed: [bool; 8],
        pub released: [bool; 8],
        pub down: [bool; 8],
    }

    impl MouseState {
        pub fn update(&mut self, rl: Raylib) {
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
    use super::{
        components,
        components::{Dim2D, Pos2D},
        resources,
    };
    use raylib::prelude::*;
    use specs::{prelude::*, storage::HashMapStorage, WorldExt};
    use std::{
        ops::{Deref, DerefMut},
        sync::{Arc, Mutex},
    };

    // -----------------------------------------------------------------------------

    enum SelectorState {
        Idle,
        Selecting(Entity, Pos2D),
        Confirmed(Entity, Pos2D, Dim2D),
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
            ReadExpect<'a, super::Camera>,
            ReadExpect<'a, resources::MouseState>,
            ReadStorage<'a, components::Pos3D>,
            WriteStorage<'a, components::Pos2D>,
            WriteStorage<'a, components::Dim2D>,
        );

        fn run(&mut self, sys_data: Self::SystemData) {
            let (entities, cam, mouse_state, positions, mut pos2Ds, mut dim2Ds) = sys_data;
            match self.state {
                SelectorState::Idle => {
                    if mouse_state.pressed[0] {
                        let pos: Pos2D = mouse_state.pos.into();
                        let e = entities.build_entity().with(pos, &mut pos2Ds).build();
                        self.state = SelectorState::Selecting(e, pos);
                    }
                }
                SelectorState::Selecting(e, upper_left @ Pos2D(x1, y1)) => {
                    let pos: Pos2D = mouse_state.pos.into();
                    let dim = Dim2D(pos.0 - x1, pos.1 - y1);
                    dim2Ds.insert(e, dim).unwrap();
                    if mouse_state.released[0] {
                        self.state = SelectorState::Confirmed(e, upper_left, dim);
                    }
                }
                SelectorState::Confirmed(e, _, _) => {
                    entities.delete(e).unwrap();
                    self.state = SelectorState::Idle;
                }
            }
        }
    }

    // -----------------------------------------------------------------------------

    pub struct Renderer;

    impl<'a> System<'a> for Renderer {
        type SystemData = (
            WriteExpect<'a, resources::Raylib>,
            ReadExpect<'a, super::Camera>,
            ReadStorage<'a, components::Pos3D>,
            ReadStorage<'a, components::Pos2D>,
            ReadStorage<'a, components::Dim2D>,
        );

        fn run(&mut self, (mut rl, cam, pos3Ds, pos2Ds, dim2Ds): Self::SystemData) {
            let thread: RaylibThread = unsafe { std::mem::transmute(()) };
            rl.draw(&thread, |d| {
                d.clear_background(Color::DARKGRAY);

                {
                    let mut d2 = d.begin_mode_3D(cam.deref().inner);
                    for pos in pos3Ds.join() {
                        d2.draw_cube(pos.0, 2.0, 2.0, 2.0, Color::RED);
                        d2.draw_cube_wires(pos.0, 2.0, 2.0, 2.0, Color::BLACK);
                    }
                }

                for (&Pos2D(x, y), &Dim2D(w, h)) in (&pos2Ds, &dim2Ds).join() {
                    d.draw_rectangle(x, y, w, h, Color::GREEN.fade(0.1));
                    d.draw_rectangle_lines(x, y, w, h, Color::GREEN);
                }
            });
        }
    }
}

// -----------------------------------------------------------------------------

// TODO(cmc): Camera should be a proper system (via resource).
// TODO(cmc): Delta should be a resource (DeltaTime).
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

    // https://gamedev.stackexchange.com/a/159314
    // TODO(cmc): should be bitsets obviously
    pub fn update(&mut self, delta: f32, (l, u, r, d): (bool, bool, bool, bool), zoom: i32) {
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

        self.y_rad += zoom as f32 * delta * 10. * Self::PI / 180.0;
        self.y_rad = self.y_rad.max(-Self::PI / 3.0).min(-Self::PI * 0.15);
        self.inner.position.y = Self::Y * 2. * self.y_rad.abs();

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

// TODO(cmc): do this properly.
pub struct RawDrawHandle(*mut ());
unsafe impl Send for RawDrawHandle {}
unsafe impl Sync for RawDrawHandle {}

impl RawDrawHandle {
    pub fn lol(&mut self) -> &mut RaylibDrawHandle {
        unsafe { std::mem::transmute(&mut self.0) }
    }
}

fn main() {
    let (rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("RTS")
        .build();

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Selector::default(), "selector", &[])
        .with_thread_local(systems::Renderer {})
        .build();
    dispatcher.setup(&mut world);

    let mut rl = resources::Raylib::new(rl);
    world.insert(rl.clone());

    use resources::MouseState;
    world.insert(MouseState::default());

    let mut cam = {
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

        let mut cam = Camera::new(inner);
        cam.update(0.0, (false, false, false, false), 0);

        cam
    };
    world.insert(cam.clone());

    for x in -10..=10 {
        for z in -10..=10 {
            let pos: Vector3 = (x as f32 * 4.0, 0.0, z as f32 * 4.0).into();
            world.create_entity().with(components::Pos3D(pos)).build();
        }
    }

    while !rl.read(|rl| rl.window_should_close()) {
        let delta = rl.read(|rl| rl.get_frame_time() * 50.0);
        let (swidth, sheight) = (rl.read(|rl| rl.get_screen_width() - 100), 40);

        world.write_resource::<MouseState>().update(rl.clone());

        rl.read(|rl| {
            cam.update(
                delta,
                (
                    rl.is_key_down(KeyboardKey::KEY_A),
                    rl.is_key_down(KeyboardKey::KEY_W),
                    rl.is_key_down(KeyboardKey::KEY_D),
                    rl.is_key_down(KeyboardKey::KEY_S),
                ),
                rl.get_mouse_wheel_move(),
            )
        });

        world.insert(cam.clone());

        dispatcher.dispatch(&mut world);
        world.maintain();

        rl.draw(&thread, |d| {
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

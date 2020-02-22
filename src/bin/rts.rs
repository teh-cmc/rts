#![feature(bindings_after_at, type_name_of_val)]

use raylib::prelude::*;
use rts::{components, resources, systems};
use specs::{prelude::*, WorldExt};

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
        let bounding_tree = systems::BoundingTree::default();
        let bounding_tree_id = sys_id(&bounding_tree);
        let renderer = systems::Renderer::default();

        DispatcherBuilder::new()
            .with(mouse, mouse_id, &[])
            .with(cam, cam_id, &[])
            .with(bounding_tree, bounding_tree_id, &[])
            .with(selector, selector_id, &[mouse_id, bounding_tree_id])
            .with_thread_local(renderer)
            .build()
    };
    dispatcher.setup(&mut world);

    let mut rl = resources::Raylib::new(rl);
    world.insert(rl.clone());

    world.insert(resources::DeltaTime(0.0));
    world.insert(resources::MouseState::default());
    world.insert(resources::BoundingTree::new());

    let cam = {
        let inner = Camera3D::perspective(
            Vector3::zero(),
            Vector3::zero(),
            (0.0, 1.0, 0.0).into(),
            60.0,
        );

        rl.write(|rl| {
            rl.set_camera_mode(&inner, CameraMode::CAMERA_CUSTOM);
        });

        resources::Camera::new(inner)
    };
    world.insert(cam);

    use components::Vec3D;
    for x in -10..=10 {
        for z in -10..=10 {
            let pos: Vec3D = (x as f32 * 4.0, 0.0, z as f32 * 4.0).into();
            let dim: Vec3D = (2.0, 2.0, 2.0).into();
            world
                .create_entity()
                .with(components::Pos3D(pos))
                .with(components::Dim3D(dim))
                .with(components::Pos3DInvalidated)
                .build();
        }
    }

    #[cfg(target_os = "emscripten")]
    unsafe {
        // TODO(cmc): Not sure why but hours of debugging have shown that I need
        // to yield back one time to the browser event-loop before gettings the
        // real stuff going...
        emscripten::emscripten_sleep(1);

        let mut main_loop = move || {
            let delta = rl.read(|rl| rl.get_frame_time() * 50.0);
            world.write_resource::<resources::DeltaTime>().0 = delta;

            dispatcher.dispatch(&mut world);
            world.maintain();
        };
        let (callback, args) = emscripten::trampoline(&mut main_loop);
        emscripten::emscripten_set_main_loop_arg(callback, args, 0, 1);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        rl.write(|rl| rl.set_target_fps(120));
        while !rl.read(|rl| rl.window_should_close()) {
            let delta = rl.read(|rl| rl.get_frame_time() * 50.0);
            world.write_resource::<resources::DeltaTime>().0 = delta;

            dispatcher.dispatch(&mut world);
            world.maintain();
        }
    }
}

// -----------------------------------------------------------------------------

#[cfg(target_os = "emscripten")]
mod emscripten {
    use std::ffi::c_void;

    type EmscriptenCallback = extern "C" fn(*mut c_void);
    type EmscriptenCallbackArgs = *mut c_void;

    extern "C" {
        pub fn emscripten_set_main_loop(callback: EmscriptenCallback, fps: isize, loopy: isize);
        pub fn emscripten_set_main_loop_arg(
            callback: EmscriptenCallback,
            arg: EmscriptenCallbackArgs,
            fps: isize,
            loopy: isize,
        );
        pub fn emscripten_sleep(ms: usize);
    }

    pub unsafe fn trampoline<F>(closure: &mut F) -> (EmscriptenCallback, EmscriptenCallbackArgs)
    where
        F: FnMut(),
    {
        extern "C" fn trampoline<F>(args: EmscriptenCallbackArgs)
        where
            F: FnMut(),
        {
            let closure: &mut F = unsafe { &mut *(args as *mut F) };
            (*closure)();
        }

        (trampoline::<F>, closure as *mut F as EmscriptenCallbackArgs)
    }
}

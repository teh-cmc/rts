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
            rl.hide_cursor();
        });

        resources::Camera::new(inner)
    };
    world.insert(cam.clone());

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

    // rl.write(|rl| {
    //     let mut proj = hacks::get_matrix_projection();
    //     proj.m0 = 0.974279;
    //     proj.m1 = 0.000000;
    //     proj.m2 = 0.000000;
    //     proj.m3 = 0.000000;
    //     proj.m4 = 0.000000;
    //     proj.m5 = 1.732051;
    //     proj.m6 = 0.000000;
    //     proj.m7 = 0.000000;
    //     proj.m8 = 0.000000;
    //     proj.m9 = 0.000000;
    //     proj.m10 = -1.000020;
    //     proj.m11 = -1.000000;
    //     proj.m12 = 0.000000;
    //     proj.m13 = 0.000000;
    //     proj.m14 = -0.020000;
    //     proj.m15 = 0.000000;
    //     rl.set_matrix_projection(&unsafe { std::mem::transmute(()) }, proj);
    // });

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

            continue;

            let (x, y) = rl.read(|rl| (rl.get_mouse_x() as f32, rl.get_mouse_y() as f32));
            let (swidth, sheight) =
                rl.read(|rl| (rl.get_screen_width() as f32, rl.get_screen_height() as f32));
            let (x, y) = (x / swidth, y / sheight);

            use cgmath::{Matrix4 as CGMat4, Vector4 as CGVec4};
            use raylib::core::math::*;
            let proj = hacks::get_matrix_projection();
            let modelview = rl.read(|rl| rl.get_matrix_modelview());
            let mat = (proj * modelview).inverted();
            dbg!(mat);
            let v = Vector4::new(x, y, 1.0, 1.0);

            let xxx = mat.to_array();
            let mat = CGMat4::new(
                xxx[0], xxx[1], xxx[2], xxx[3], xxx[4], xxx[5], xxx[6], xxx[7], xxx[8], xxx[9],
                xxx[10], xxx[11], xxx[12], xxx[13], xxx[14], xxx[15],
            );

            let v = CGVec4::new(v.x, v.y, v.z, v.w);
            let v = mat * v;
            let v = v / v.w;

            dbg!(v);
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

// -----------------------------------------------------------------------------

mod hacks {
    use raylib::{core::math::Matrix, ffi::Matrix as c_matrix};

    extern "C" {
        fn GetMatrixProjection() -> c_matrix;
        fn GetMatrixModelview() -> c_matrix;
    }

    pub fn get_matrix_projection() -> Matrix {
        unsafe { GetMatrixProjection().into() }
    }
    pub fn get_matrix_modelview() -> Matrix {
        unsafe { GetMatrixModelview().into() }
    }
}

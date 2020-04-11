#![feature(bindings_after_at, type_name_of_val)]

use raylib::prelude::*;
use rts::{components::prelude::*, maths::prelude::*, resources::prelude::*, systems::prelude::*};
use specs::{prelude::*, WorldExt};
use std::sync::Arc;

// -----------------------------------------------------------------------------

fn main() {
    const WINDOW_WIDTH: i32 = 1280;
    const WINDOW_HEIGHT: i32 = 720;
    let (rl, rl_thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("RTS")
        .build();

    let mut world = World::new();

    let mut dispatcher = {
        // TODO(cmc): macro this?
        use std::any::type_name_of_val as sys_id;
        let mouse = SysMouse::default();
        let mouse_id = sys_id(&mouse);
        let cam = SysCamera::default();
        let cam_id = sys_id(&cam);
        let selector = SysSelector::default();
        let selector_id = sys_id(&selector);
        let bounding_tree = SysBoundingTree::default();
        let bounding_tree_id = sys_id(&bounding_tree);
        let renderer = SysRenderer::new(rl_thread.clone());

        DispatcherBuilder::new()
            .with(mouse, mouse_id, &[])
            .with(cam, cam_id, &[])
            .with(bounding_tree, bounding_tree_id, &[])
            .with(selector, selector_id, &[mouse_id, bounding_tree_id])
            .with_thread_local(renderer)
            .build()
    };
    dispatcher.setup(&mut world);

    world.insert(ResrcDeltaTime(0.0));
    world.insert(ResrcMouseState::default());
    world.insert(ResrcBoundingTree::new());
    world.insert(ResrcModelView::default());
    world.insert(ResrcProjection::default());

    let models = rts::voxel::VoxelModel::from_vox(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/submodules/voxel-model/vox/scan/dragon.vox"
    )))
    .unwrap();
    for model in models.into_iter() {
        world
            .create_entity()
            .with(CompVoxelModel(model))
            .with(CompGridPosition((0, 0, 0).into()))
            .with(CompInvalidated)
            .with(CompColor(Color::RED))
            .build();
    }

    // world
    //     .create_entity()
    //     .with(CompVoxelModel(rts::voxel::VoxelModel::checkerboard()))
    //     .with(CompGridPosition((0, 0, 0).into()))
    //     .with(CompInvalidated)
    //     .with(CompColor(Color::RED))
    //     .build();

    // const TEAPOT_PATH: &str =
    // "/home/cmc/dev/ephtracy/voxel-model/vox/scan/teapot.vox"; let meshes =
    // ResrcMeshStore::new(&rl_thread); vox::load(&mut rl, &rl_thread, &meshes,
    // &mut world, TEAPOT_PATH).unwrap(); let tex = raylib::core::texture::
    // Image::gen_image_color(1, 1, Color::WHITE); let mut tex =
    // rl.load_texture_from_image(&rl_thread, &tex).unwrap();
    // tex.gen_texture_mipmaps();
    // for x in -10..=10 {
    //     for z in -10..=10 {
    //         let cube = meshes.instantiate_model(&mut rl, &rl_thread,
    // ResrcMeshStore::CUBE, &tex);         let cube =
    // CompModel3D(Arc::new(cube));         let transform =
    // CGMat4::from_translation((x as f32 * 2.0, 0.0, z as f32 * 2.0).into());
    //         let transform = transform * CGMat4::from_scale(2.);
    //         world
    //             .create_entity()
    //             .with(CompTransform3D(transform.into()))
    //             .with(cube)
    //             .with(CompInvalidated)
    //             .with(CompColor(Color::RED))
    //             .build();
    //     }
    // }
    // world.insert(meshes);

    let mut rl = ResrcRaylib::new(rl);
    world.insert(rl.clone());
    rl.write(|rl| rl.hide_cursor());

    let cam = {
        let inner = Camera3D::perspective(
            Vector3::zero(),
            Vector3::zero(),
            (0.0, 1.0, 0.0).into(),
            60.0,
        );

        ResrcCamera::new(inner, ResrcCameraMode::RTS)
    };
    world.insert(cam);

    #[cfg(target_os = "emscripten")]
    unsafe {
        // TODO(cmc): Not sure why but hours of debugging have shown that I need
        // to yield back one time to the browser event-loop before getting the
        // real stuff going...
        emscripten::emscripten_sleep(1);

        let mut main_loop = move || {
            let delta = rl.read(|rl| rl.get_frame_time());
            world.write_resource::<ResrcDeltaTime>().0 = delta;

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
            let delta = rl.read(|rl| rl.get_frame_time());
            world.write_resource::<ResrcDeltaTime>().0 = delta;

            dispatcher.dispatch(&mut world);
            world.maintain();
        }
    }
}

// -----------------------------------------------------------------------------

mod vox {
    use super::*;
    use anyhow::{anyhow, Error as AnyError, Result as AnyResult};
    use std::sync::Arc;

    pub fn load(
        rl: &mut RaylibHandle,
        rl_thread: &RaylibThread,
        meshes: &ResrcMeshStore,
        world: &mut World,
        path: &str,
    ) -> AnyResult<()> {
        let tex = raylib::core::texture::Image::gen_image_color(1, 1, Color::WHITE);
        let mut tex = rl.load_texture_from_image(&rl_thread, &tex).unwrap();
        tex.gen_texture_mipmaps();
        let cube = meshes.instantiate_model(rl, rl_thread, ResrcMeshStore::CUBE, &tex);
        let cube = Arc::new(cube);

        let data = dot_vox::load(path).map_err(|msg| anyhow!("{}", msg))?;
        for model in data.models {
            dbg!(model.voxels.len());
            for voxel in model.voxels {
                let cube = CompModel3D(Arc::clone(&cube));
                let transform = CGMat4::from_translation(
                    (voxel.x as f32, voxel.y as f32, voxel.z as f32).into(),
                );
                world
                    .create_entity()
                    .with(CompTransform3D(transform.into()))
                    .with(cube)
                    .with(CompInvalidated)
                    .with(CompColor(Color::RED))
                    .build();
            }
        }
        Ok::<_, AnyError>(())
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

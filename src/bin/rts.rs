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

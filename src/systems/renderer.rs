use crate::{components, resources};
use raylib::prelude::*;
use specs::prelude::*;

// -----------------------------------------------------------------------------

#[derive(Default)]
pub struct Renderer;

impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, resources::Raylib>,
        ReadExpect<'a, resources::Camera>,
        Entities<'a>,
        ReadStorage<'a, components::Pos3D>,
        ReadStorage<'a, components::Dim3D>,
        ReadStorage<'a, components::Pos2D>,
        ReadStorage<'a, components::Dim2D>,
        ReadStorage<'a, components::Selected>,
    );

    fn run(&mut self, sys_data: Self::SystemData) {
        let (mut rl, cam, entities, pos3Ds, dim3Ds, pos2Ds, dim2Ds, selected) = sys_data;

        // TODO(cmc): safety note.
        let thread: RaylibThread = unsafe { std::mem::transmute(()) };
        let (swidth, sheight) = (rl.read(|rl| rl.get_screen_width() - 100), 40);
        let (x, y) = rl.read(|rl| (rl.get_mouse_x() as f32, rl.get_mouse_y() as f32));
        rl.draw(&thread, |d| {
            d.clear_background(Color::DARKGRAY);

            {
                let mut d2 = d.begin_mode_3D(cam.raw());
                for (e, &pos, &dim) in (&entities, &pos3Ds, &dim3Ds).join() {
                    let pos: Vector3 = pos.into();
                    let dim: Vector3 = dim.into();
                    let col = if let Some(_) = selected.get(e) {
                        Color::GOLD
                    } else {
                        Color::RED
                    };
                    // NOTE(cmc): Raylib draws cube from their center of
                    // gravity, not their corner, i.e.:
                    // rlVertex3f(x + width/2, y + height/2, z - length/2);
                    // rlVertex3f(x + width/2, y - height/2, z - length/2);
                    // rlVertex3f(x - width/2, y + height/2, z - length/2);
                    //
                    // Hence `pos + dim / 2.0`.
                    d2.draw_cube(pos + dim / 2.0, dim.x, dim.y, dim.z, col);
                    d2.draw_cube_wires(pos + dim / 2.0, dim.x, dim.y, dim.z, Color::BLACK);
                }
            }

            use components::{Dim2D, Pos2D};
            for (&Pos2D(pos), &Dim2D(dim)) in (&pos2Ds, &dim2Ds).join() {
                d.draw_rectangle(pos.x, pos.y, dim.x, dim.y, Color::GREEN.fade(0.1));
                d.draw_rectangle_lines(pos.x, pos.y, dim.x, dim.y, Color::GREEN);
            }

            // Poor man's cursor
            {
                use raylib::core::math::Vector2 as RayVector2;
                d.draw_circle_sector(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::SKYBLUE);
                d.draw_circle_sector_lines(RayVector2::new(x, y), 25.0, 30, 60, 1, Color::BLUE);
            }

            d.draw_rectangle(10, 10, 220, 70, Color::SKYBLUE);
            d.draw_rectangle_lines(10, 10, 220, 70, Color::BLUE);
            d.draw_text("Camera default controls:", 20, 20, 10, Color::BLACK);
            d.draw_text("- Move with keys: W, A, S, D", 40, 40, 10, Color::DARKGRAY);
            d.draw_text("- Mouse wheel to zoom", 40, 60, 10, Color::DARKGRAY);

            d.draw_fps(swidth, sheight);
        });
    }
}

// -----------------------------------------------------------------------------

mod hacks {
    use raylib::{core::math::Matrix, ffi::Matrix as c_matrix};

    extern "C" {
        fn GetMatrixProjection() -> c_matrix;
    }

    pub fn get_matrix_projection() -> Matrix {
        unsafe { GetMatrixProjection().into() }
    }
}

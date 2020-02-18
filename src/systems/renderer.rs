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

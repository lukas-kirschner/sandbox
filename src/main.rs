//   sandbox - A simple 2D physics game
//   Copyright (C) 2025  Lukas Kirschner
//
//   This program is free software: you can redistribute it and/or modify
//   it under the terms of the GNU General Public License as published by
//   the Free Software Foundation, either version 3 of the License, or
//   (at your option) any later version.
//
//   This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU General Public License for more details.
//
//   You should have received a copy of the GNU General Public License
//   along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod colors;
mod element;
mod ui;
mod world;
/// How fast the simulation runs, independently of framerate
const TICKS_PER_SECOND: usize = 120;

use std::time::Instant;
use crate::element::Element;
use crate::ui::Ui;
use crate::world::GameWorld;
use egui_sdl2_gl::{with_sdl2, EguiStateHandler};
use egui_sdl2_gl::{egui, DpiScaling, ShaderVersion};
use egui_sdl2_gl::egui::{Context, FullOutput};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::GLProfile;
fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    // On linux, OpenGL ES Mesa driver 22.0.0+ can be used like so:
    // gl_attr.set_context_profile(GLProfile::GLES);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
    let ui = Ui::new(1280, 720);
    let window = video_subsystem
        .window("Sandbox", ui.win_width as u32, ui.win_height as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let _ctx = window.gl_create_context()?;
    let shader_ver = ShaderVersion::Default;
    let (mut painter, mut egui_state) = with_sdl2(&window, shader_ver, DpiScaling::Default);
    let egui_ctx = egui::Context::default();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let mut world: GameWorld = GameWorld::new(ui.board_width, ui.board_height);
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(
            PixelFormatEnum::ARGB8888,
            ui.board_width as u32,
            ui.board_height as u32,
        )
        .unwrap();
    let timer = sdl_context.timer()?;
    let mut prev_tick = timer.ticks64();
    let mut rng = XorShiftRng::seed_from_u64(0);
    let mut event_pump = sdl_context.event_pump()?;
    let start_time = Instant::now();
    'running: loop {
        draw_ui(&mut egui_state, &start_time, &egui_ctx);
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseMotion {
                    mousestate, x, y, ..
                } => {
                    if mousestate.is_mouse_button_pressed(MouseButton::Left) {
                        world.insert_element_at(&ui, x, y, Element::Sand);
                    } else if mousestate.is_mouse_button_pressed(MouseButton::Right) {
                        // Delete the element at the given position
                        world.insert_element_at(&ui, x, y, Element::None);
                    } else {
                        // world.show_element_preview(&ui,x,y,Element::Sand);
                    }
                },
                _ => {},
            }
        }
        let ticks =
            ((timer.ticks64() - prev_tick) as f64 / 1000. * TICKS_PER_SECOND as f64) as usize;
        if ticks > 0 {
            for _ in 0..ticks {
                // Calculate the next board state
                world.tick(&mut rng);
            }
            prev_tick = timer.ticks64();
        }
        // Draw the new board to the window
        ui.draw(&mut canvas, &mut texture, &world)?;

        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = egui_ctx.end_pass();
        egui_state.process_output(&canvas.window(), &platform_output);
        let paint_jobs = egui_ctx.tessellate(shapes, pixels_per_point);
        painter.paint_jobs(None, textures_delta, paint_jobs);

        // Update the window
        canvas.present();
    }
    Ok(())
}

fn draw_ui(egui_state:&mut EguiStateHandler, start_time:&Instant, egui_ctx: &Context) {
    egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
    egui_ctx.begin_pass(egui_state.input.take());
    egui::panel::SidePanel::right("sel-buttons").show(egui_ctx,|ui| {
        ui.button("X");
    });
}

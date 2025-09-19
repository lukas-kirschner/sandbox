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

use crate::element::Element;
use crate::ui::Ui;
use crate::world::GameWorld;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;
use sdl2::video::GLProfile;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let gl_attr = video_subsystem.gl_attr();
    // gl_attr.set_context_profile(GLProfile::Core);
    // On linux, OpenGL ES Mesa driver 22.0.0+ can be used like so:
    gl_attr.set_context_profile(GLProfile::GLES);
    let ui = Ui::new(1280, 720);
    let window = video_subsystem
        .window("Sandbox", ui.win_width as u32, ui.win_height as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
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
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
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
                world = world.tick();
            }
            prev_tick = timer.ticks64();
        }
        // Draw the new board to the window
        ui.draw(&mut canvas, &mut texture, &world)?;
        // Update the window
        canvas.present();
    }
    Ok(())
}

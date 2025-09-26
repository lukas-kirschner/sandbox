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
use imgui::{Condition, StyleColor};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;
use strum::IntoEnumIterator;

pub const FONT_SIZE: f32 = 13.0;

/// Creates the imgui context. Stolen from https://github.com/imgui-rs/imgui-examples/blob/main/examples/support/mod.rs
// pub fn create_context() -> imgui::Context {
//     let mut imgui = Context::create();
//     // Fixed font size. Note imgui_winit_support uses "logical
//     // pixels", which are physical pixels scaled by the devices
//     // scaling factor. Meaning, 13.0 pixels should look the same size
//     // on two different screens, and thus we do not need to scale this
//     // value (as the scaling is handled by winit)
//     imgui.fonts().add_font(&[
//         FontSource::TtfData {
//             data: include_bytes!("../../resources/Roboto-Regular.ttf"),
//             size_pixels: FONT_SIZE,
//             config: Some(FontConfig {
//                 // As imgui-glium-renderer isn't gamma-correct with
//                 // it's font rendering, we apply an arbitrary
//                 // multiplier to make the font a bit "heavier". With
//                 // default imgui-glow-renderer this is unnecessary.
//                 rasterizer_multiply: 1.5,
//                 // Oversampling font helps improve text rendering at
//                 // expense of larger font atlas texture.
//                 oversample_h: 4,
//                 oversample_v: 4,
//                 ..FontConfig::default()
//             }),
//         },
//         FontSource::TtfData {
//             data: include_bytes!("../../resources/mplus-1p-regular.ttf"),
//             size_pixels: FONT_SIZE,
//             config: Some(FontConfig {
//                 // Oversampling font helps improve text rendering at
//                 // expense of larger font atlas texture.
//                 oversample_h: 4,
//                 oversample_v: 4,
//                 // Range of glyphs to rasterize
//                 glyph_ranges: FontGlyphRanges::japanese(),
//                 ..FontConfig::default()
//             }),
//         },
//     ]);
//     imgui.set_ini_filename(None);
//
//     imgui
// }

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
    }
    let game_world = Ui::new(1280, 720);
    let window = video_subsystem
        .window(
            "Sandbox",
            game_world.win_width as u32,
            game_world.win_height as u32,
        )
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    let mut imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let mut world: GameWorld = GameWorld::new(game_world.board_width, game_world.board_height);
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(
            PixelFormatEnum::ARGB8888,
            game_world.board_width as u32,
            game_world.board_height as u32,
        )
        .unwrap();
    let mut rng = XorShiftRng::seed_from_u64(0);
    let mut current_elem = Element::Sand;
    let mut event_pump = sdl_context.event_pump()?;
    let mut last_frame = Instant::now();
    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            imgui_sdl.handle_event(&mut imgui, &event);
            if imgui_sdl.ignore_event(&event) {
                continue;
            }
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
                        world.insert_element_at(&game_world, x, y, Element::Sand);
                    } else if mousestate.is_mouse_button_pressed(MouseButton::Right) {
                        // Delete the element at the given position
                        world.insert_element_at(&game_world, x, y, Element::None);
                    } else {
                        // world.show_element_preview(&ui,x,y,Element::Sand);
                    }
                },
                _ => {},
            }
        }
        imgui_sdl.prepare_frame(imgui.io_mut(), canvas.window(), &event_pump.mouse_state());

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui.io_mut().delta_time = delta_s;

        // let no_ticks = TICKS_PER_SECOND as f32 * delta_s;
        world.tick(&mut rng);

        let ui = imgui.frame();
        build_element_buttons(&ui, &game_world, &mut current_elem);

        // Update the window graphics
        // Draw the new board to the window
        game_world.draw(&mut canvas, &mut texture, &world)?;
        unsafe { gl::Flush() };
        canvas.window_mut().gl_make_current(&_gl_context);
        imgui_sdl.prepare_render(&ui, canvas.window());
        renderer.render(&mut imgui);
        // Flush the GL buffer. Workaround for white windows
        unsafe { gl::Flush() };
        canvas.present();
        // canvas.window().gl_swap_window();
    }
    Ok(())
}

fn build_element_buttons(ui: &imgui::Ui, game_world: &Ui, selected: &mut Element) {
    let [win_width, win_height] = ui.io().display_size;
    // Border width 1px
    let buttonbar_width = (game_world.win_width - game_world.board_width) / 2 - 2;
    let win = ui
        .window("element_button_sidebar")
        .size([buttonbar_width as f32, win_height], Condition::Always)
        .resizable(false)
        .movable(false)
        .position([win_width - buttonbar_width as f32, 0.0], Condition::Always)
        .movable(false)
        .collapsible(false)
        .title_bar(false);
    win.build(|| {
        for e in Element::iter() {
            if e == Element::None {
                continue;
            }
            let bgcolor = if &e == selected {
                ui.push_style_color(StyleColor::Button, [0.6, 0.6, 0.6, 1.0])
            } else {
                ui.push_style_color(StyleColor::Button, [0.2, 0.2, 0.2, 1.0])
            };
            ui.button(format!("{:?}", e));
            bgcolor.pop();
        }
    });
}

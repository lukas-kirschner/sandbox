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
// /// How fast the simulation runs, independently of framerate
// const TICKS_PER_SECOND: usize = 120;

// UI colors:
const INACTIVE_BUTTON_BACKGROUND: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const ACTIVE_BUTTON_BACKGROUND: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const TEXT_FOREGROUND: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const HOVERED_BUTTON_BACKGROUND: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
// const TOOLTIP_TEXT_DENSITY: [f32; 4] = [0.7, 0.7, 0.2, 1.0];

use crate::element::{Element, ElementKind};
use crate::ui::Ui;
use crate::world::GameWorld;
use imgui::{Condition, Context, Style, StyleColor};
use imgui_sdl2_support::SdlPlatform;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseState};
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;
use imgui_sdl2_canvas_renderer::CanvasRenderer;
use strum::IntoEnumIterator;

pub const FONT_SIZE: f32 = 13.0;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    {
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(1, 1);
    }
    let mut game_world = Ui::new(1800, 960, 4);
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
    let gl_context = window
        .gl_create_context()
        .map_err(|e| format!("Couldn't create GL context: {:?}", e))?;
    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    set_imgui_style(imgui.style_mut());
    let mut platform = SdlPlatform::new(&mut imgui);
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let mut renderer = CanvasRenderer::new(&mut imgui, &mut canvas)?;
    let mut world: GameWorld = GameWorld::new(
        game_world.board_width,
        game_world.board_height,
        game_world.scaling_factor,
    );
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
            if platform.handle_event(&mut imgui, &event) {
                continue;
            }
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {},
            }
        }
        // Always handle mouse events, no matter if the mouse is moved
        let state = MouseState::new(&event_pump);
        if state.is_mouse_button_pressed(MouseButton::Left) {
            world.insert_element_at(&game_world, state.x(), state.y(), current_elem);
        } else if state.is_mouse_button_pressed(MouseButton::Right) {
            world.insert_element_at(&game_world, state.x(), state.y(), Element::None);
        }

        platform.prepare_frame(&mut imgui, canvas.window(), &event_pump);

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui.io_mut().delta_time = delta_s;

        // let no_ticks = TICKS_PER_SECOND as f32 * delta_s;
        // Tick once for scaling 4, 4x for scaling 1
        for _ in 0..(5i32 - game_world.scaling_factor as i32).max(1) {
            world.tick(&mut rng);
        }

        let ui = imgui.new_frame();
        build_element_buttons(ui, &game_world, &mut current_elem);
        build_top_settings_pane(ui, &mut game_world);

        // Update the window graphics
        // Draw the new board to the window
        game_world.draw(&mut canvas, &mut texture, &world)?;
        game_world.draw_mouse_preview_at(&mut canvas, state.x(), state.y(), &world)?;

        // Render imgui
        canvas.window_mut().gl_make_current(&gl_context)?;
        let draw_data = imgui.render();
        renderer.render(draw_data, &mut canvas)?;

        canvas.present();
    }
    Ok(())
}

fn set_imgui_style(style: &mut Style) {
    style.use_dark_colors();
    style[StyleColor::Text] = TEXT_FOREGROUND;
    style[StyleColor::Button] = INACTIVE_BUTTON_BACKGROUND;
    style[StyleColor::ButtonHovered] = HOVERED_BUTTON_BACKGROUND;
}

fn build_element_buttons(ui: &imgui::Ui, game_world: &Ui, selected: &mut Element) {
    let [win_width, win_height] = ui.io().display_size;
    // Border width 1px
    let buttonbar_width = (game_world.win_width - game_world.board_width) / 2 - 2;
    let win = ui
        .window("element_button_sidebar")
        .size(
            [
                buttonbar_width as f32,
                win_height - (game_world.win_height - game_world.board_height + 2) as f32,
            ],
            Condition::Always,
        )
        .resizable(false)
        .movable(false)
        .position(
            [
                win_width - buttonbar_width as f32,
                (game_world.win_height - game_world.board_height) as f32 / 2. - 1.,
            ],
            Condition::Always,
        )
        .movable(false)
        .collapsible(false)
        .title_bar(false);
    win.build(|| {
        for kind in ElementKind::iter() {
            if matches!(kind, ElementKind::None) {
                continue;
            }
            ui.text(match kind {
                ElementKind::None => "",
                ElementKind::Solid => "Solids:",
                ElementKind::Powder { .. } => "Powders:",
                ElementKind::Liquid { .. } => "Liquids:",
                ElementKind::Gas { .. } => "Gases:",
            });
            ui.spacing();
            for e in Element::iter().filter(|e| e.is_kind_of(&kind)) {
                if e == Element::None || matches!(e, Element::BurningParticle { .. }) {
                    continue;
                }
                let hovercolor =
                    ui.push_style_color(StyleColor::ButtonHovered, HOVERED_BUTTON_BACKGROUND);
                let bgcolor = if &e == selected {
                    ui.push_style_color(StyleColor::Button, ACTIVE_BUTTON_BACKGROUND)
                } else {
                    ui.push_style_color(StyleColor::Button, INACTIVE_BUTTON_BACKGROUND)
                };
                ui.button(format!("{}", e));
                if ui.is_item_clicked() {
                    *selected = e;
                }
                // TODO How to draw Imgui above game board?
                // if ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                //     ui.tooltip(|| {
                //         ui.text(format!("{}", e));
                //         if let Some(density) = e.density() {
                //             ui.text_colored(
                //                 TOOLTIP_TEXT_DENSITY,
                //                 format!("density {:.2}kg/m³", density),
                //             );
                //         }
                //     });
                // }
                bgcolor.pop();
                hovercolor.pop()
            }
            ui.separator();
            ui.spacing();
        }
    });
}
fn build_top_settings_pane(ui: &imgui::Ui, game_world: &mut Ui) {
    let [win_width, _win_height] = ui.io().display_size;
    // Border width 1px
    let buttonbar_height = (game_world.win_height - game_world.board_height) / 2 - 2;
    let win = ui
        .window("element_button_settings")
        .size([win_width, buttonbar_height as f32], Condition::Always)
        .resizable(false)
        .movable(false)
        .position([0.0, 0.0], Condition::Always)
        .movable(false)
        .collapsible(false)
        .title_bar(false);
    win.build(|| {
        ui.text("Cursor:");
        for e in [1, 2, 3, 4, 5, 10, 15, 20] {
            ui.same_line();
            let hovercolor =
                ui.push_style_color(StyleColor::ButtonHovered, HOVERED_BUTTON_BACKGROUND);
            let bgcolor = if e == game_world.cursor_size() {
                ui.push_style_color(StyleColor::Button, ACTIVE_BUTTON_BACKGROUND)
            } else {
                ui.push_style_color(StyleColor::Button, INACTIVE_BUTTON_BACKGROUND)
            };
            ui.button(format!("{}", e));
            if ui.is_item_clicked() {
                game_world.set_cursor_size(e);
            }
            bgcolor.pop();
            hovercolor.pop();
        }
    });
}

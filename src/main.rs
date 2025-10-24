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
const TOOLTIP_TEXT_DENSITY: Color32 = Color32::from_rgb(0xAA, 0xAA, 0x44);
const TOOLTIP_TEXT_DESCRIPTION: Color32 = Color32::from_rgb(0x66, 0x66, 0x66);

use crate::element::{Element, ElementKind};
use crate::ui::Ui;
use crate::world::GameWorld;
use egui::FontFamily::Proportional;
use egui::{Align, Color32, FontId, Layout, RichText, TextStyle};
use egui_sdl2_canvas::Painter;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseState};
use sdl2::pixels::PixelFormatEnum;
use std::time::Instant;
use strum::IntoEnumIterator;

pub const FONT_SIZE: f32 = 13.0;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut video_subsystem = sdl_context.video()?;
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
    let mut platform = egui_sdl2_platform::Platform::new((
        game_world.win_width as u32,
        game_world.win_height as u32,
    ))
    .map_err(|e| format!("{}", e))?;
    // Set up egui style:
    platform.context().set_pixels_per_point(1.0);
    platform.context().set_visuals(egui::Visuals::dark());
    platform.context().style_mut(|style| {
        style.text_styles = [
            (TextStyle::Button, FontId::new(14.0, Proportional)),
            (TextStyle::Body, FontId::new(14.0, Proportional)),
        ]
        .into();
    });
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut painter = Painter::new();
    let mut world: GameWorld = GameWorld::new(
        game_world.board_width,
        game_world.board_height,
        game_world.scaling_factor,
    );
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

    let start_time = Instant::now();
    'running: loop {
        platform.update_time(start_time.elapsed().as_secs_f64());
        // get the inputs here
        for event in event_pump.poll_iter() {
            platform.handle_event(&event, &sdl_context, &video_subsystem);
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

        // let no_ticks = TICKS_PER_SECOND as f32 * delta_s;
        // Tick once for scaling 4, 4x for scaling 1
        for _ in 0..(5i32 - game_world.scaling_factor as i32).max(1) {
            world.tick(&mut rng);
        }

        // platform::context() has SIDE EFFECTS - Calling it twice causes button clicks to be ignored!
        let ctx = platform.context();
        build_element_buttons(&ctx, &game_world, &mut current_elem);
        build_top_settings_pane(&ctx, &mut game_world);

        let output = platform.end_frame(&mut video_subsystem).unwrap();
        let v_primitives = platform.tessellate(&output);

        // Update the window graphics
        // Draw the new board to the window
        game_world.draw(&mut canvas, &mut texture, &world)?;
        game_world.draw_mouse_preview_at(&mut canvas, state.x(), state.y(), &world)?;

        // Render imgui
        canvas.window_mut().gl_make_current(&gl_context)?;

        if let Err(err) = painter.paint_and_update_textures(
            platform.context().pixels_per_point(),
            &output.textures_delta,
            &creator,
            &v_primitives,
            &mut canvas,
        ) {
            println!("{}", err);
        }

        canvas.present();
    }
    Ok(())
}

fn build_element_buttons(context: &egui::Context, game_world: &Ui, selected: &mut Element) {
    let buttonbar_width = game_world.right_buttonbar_width();
    egui::SidePanel::right("RightPnl")
        .exact_width(buttonbar_width)
        .resizable(false)
        .show(context, |ui| {
            // Add space to avoid overlapping Top bar
            ui.add_space(game_world.top_buttonbar_height());
            for kind in ElementKind::iter() {
                if matches!(kind, ElementKind::None) {
                    continue;
                }
                ui.label(match kind {
                    ElementKind::None => "",
                    ElementKind::Solid => "Solids:",
                    ElementKind::Powder { .. } => "Powders:",
                    ElementKind::Liquid { .. } => "Liquids:",
                    ElementKind::Gas { .. } => "Gases:",
                });
                ui.add_space(ui.spacing().item_spacing.y);
                for e in Element::iter().filter(|e| e.is_kind_of(&kind)) {
                    if !e.show_in_ui() {
                        continue;
                    }
                    let mut is_selected = &e == selected;
                    let tv = ui.toggle_value(&mut is_selected, format!("{}", e));
                    if tv.clicked() && is_selected {
                        *selected = e;
                    }
                    tv.on_hover_ui_at_pointer(|ui| {
                        ui.label(format!("{}", e));
                        if e.show_density() {
                            if let Some(density) = e.density() {
                                ui.colored_label(
                                    TOOLTIP_TEXT_DENSITY,
                                    format!("density {:.2}kg/m³", density),
                                );
                            }
                        }
                        // Show wrapped text as description:
                        ui.set_max_width(200.0);
                        ui.separator();
                        ui.style_mut().wrap = Some(true);
                        ui.label(RichText::new(e.ui_description()).color(TOOLTIP_TEXT_DESCRIPTION));
                    });
                }
                ui.separator();
                ui.add_space(ui.spacing().item_spacing.y);
            }
        });
}
fn build_top_settings_pane(context: &egui::Context, game_world: &mut Ui) {
    let buttonbar_height =
        game_world.top_buttonbar_height() - context.style().spacing.window_margin.top;
    egui::TopBottomPanel::top("TopPnl")
        .resizable(false)
        .exact_height(buttonbar_height)
        .show(context, |ui| {
            // Add top margin
            ui.add_space(ui.spacing().item_spacing.y * 2.);
            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                // Start items at left board edge
                ui.add_space(game_world.left_buttonbar_width());
                ui.label("Cursor:");
                for e in [1, 2, 3, 4, 5, 10, 15, 20] {
                    let mut sel = e == game_world.cursor_size();
                    let tv = ui.toggle_value(&mut sel, format!("{}", e));
                    if tv.clicked() && sel {
                        game_world.set_cursor_size(e);
                    }
                    tv.on_hover_text_at_pointer(format!(
                        "Set the cursor size to a {}x{} square",
                        e, e
                    ));
                }
            });
        });
}

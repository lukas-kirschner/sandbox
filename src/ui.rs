use crate::colors::BOARD_BORDER_COLOR;
use crate::world::GameWorld;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;

pub struct Ui {
    pub win_width: usize,
    pub win_height: usize,
    pub board_width: usize,
    pub board_height: usize,
}

impl Ui {
    pub fn cursor_size(&self) -> i32 {
        1
    }
}

impl Ui {
    pub(crate) fn window_to_board_coordinate(
        &self,
        window_x: i32,
        window_y: i32,
    ) -> Option<(i32, i32)> {
        let ret = (
            window_x - (self.win_width - self.board_width) as i32 / 2,
            window_y - (self.win_height - self.board_height) as i32 / 2,
        );
        if ret.0 < 0 || ret.1 < 0 || ret.0 >= self.board_width as i32 || ret.1 >= self.board_height as i32 {
            None
        } else {
            Some(ret)
        }
    }
}

impl Ui {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            win_width: width,
            win_height: height,
            board_width: width - 120,
            board_height: height - 40,
        }
    }
    /// Draw the window content
    pub fn draw(&self, canvas: &mut WindowCanvas, world: &GameWorld) -> Result<(), String> {
        let left_padding: i32 = ((self.win_width - self.board_width) / 2) as i32 - 1;
        let top_padding: i32 = ((self.win_height - self.board_height) / 2) as i32 - 1;
        let right_padding: i32 = ((self.win_width as i32 - left_padding)
            + ((self.win_width - self.board_width) % 2) as i32);
        let bottom_padding: i32 = ((self.win_height as i32 - top_padding)
            + ((self.win_height - self.board_height) % 2) as i32);

        // Draw a border around the board
        canvas.set_draw_color(BOARD_BORDER_COLOR);
        canvas.draw_line(
            Point::from((left_padding, top_padding)),
            Point::from((right_padding, top_padding)),
        )?;
        canvas.draw_line(
            Point::from((right_padding, top_padding)),
            Point::from((right_padding, bottom_padding)),
        )?;
        canvas.draw_line(
            Point::from((right_padding, bottom_padding)),
            Point::from((left_padding, bottom_padding)),
        )?;
        canvas.draw_line(
            Point::from((left_padding, bottom_padding)),
            Point::from((left_padding, top_padding)),
        )?;
        // Draw the board
        for y in 0..self.board_height {
            for x in 0..self.board_width {
                canvas.set_draw_color(world.board()[x][y].color());
                canvas.draw_point(Point::from((
                    x as i32 + left_padding + 1,
                    y as i32 + top_padding + 1,
                )))?;
            }
        }

        // Finalize
        canvas.present();
        Ok(())
    }
}

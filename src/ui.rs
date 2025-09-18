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
        for y in 0..self.board_height{
            for x in 0..self.board_width{
                canvas.set_draw_color(world.board()[x][y].color());
                canvas.draw_point(Point::from((x as i32 + left_padding + 1,y as i32 + top_padding + 1)))?;
            }
        }

        // Finalize
        canvas.present();
        Ok(())
    }
}

use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::raw::RawU32;
use embedded_graphics::prelude::{Dimensions, DrawTarget, PixelColor, Point, Size};
use embedded_graphics::primitives::Rectangle;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas, RenderTarget};

pub struct CanvasDisplay<'a, T>
where
    T: RenderTarget,
{
    pub canvas: &'a mut Canvas<T>,
    pub width: usize,
    pub height: usize,
    pub left_padding: i32,
    pub right_padding: i32,
    pub top_padding: i32,
    pub bottom_padding: i32,
}
impl<'a, T> Dimensions for CanvasDisplay<'a, T>
where
    T: RenderTarget,
{
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(self.width as u32, self.height as u32),
        )
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Rgba8888Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl From<Color> for Rgba8888Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}
impl From<Rgba8888Color> for Color {
    fn from(val: Rgba8888Color) -> Self {
        Color::RGBA(val.r, val.g, val.b, val.a)
    }
}
impl PixelColor for Rgba8888Color {
    type Raw = RawU32;
}
impl<'a, T> DrawTarget for CanvasDisplay<'a, T>
where
    T: RenderTarget,
{
    type Color = Rgba8888Color;
    type Error = String;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.canvas.set_draw_color::<Color>(pixel.1.into());
            self.canvas.set_blend_mode(BlendMode::Blend);
            if pixel.0.x > self.left_padding
                && pixel.0.x < (self.width as i32 - self.right_padding)
                && pixel.0.y > self.top_padding
                && pixel.0.y < (self.height as i32 - self.bottom_padding)
            {
                self.canvas.draw_point((pixel.0.x, pixel.0.y))?;
            }
        }
        Ok(())
    }
}

use crate::display::driver::LedCanvas;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Size,
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

pub struct EmbeddedGraphicsCanvas<'a> {
    canvas: &'a mut Box<dyn LedCanvas>,
}

impl<'a> EmbeddedGraphicsCanvas<'a> {
    pub fn new(canvas: &'a mut Box<dyn LedCanvas>) -> Self {
        Self { canvas }
    }

    // Add a method to get mutable access to the underlying canvas
    pub fn inner_mut(&mut self) -> &mut Box<dyn LedCanvas> {
        &mut self.canvas
    }
}

impl<'a> DrawTarget for EmbeddedGraphicsCanvas<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            // Only draw pixels within bounds
            if point.x >= 0 && point.y >= 0 {
                let x = point.x as usize;
                let y = point.y as usize;
                
                // Use the method call syntax directly on the color object
                self.canvas.set_pixel(x, y, color.r(), color.g(), color.b());
            }
        }
        Ok(())
    }

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        // Use black for clear
        self.canvas.fill(0, 0, 0);
        Ok(())
    }
}

impl<'a> embedded_graphics::prelude::OriginDimensions for EmbeddedGraphicsCanvas<'a> {
    fn size(&self) -> Size {
        let (width, height) = self.canvas.size();
        Size::new(width as u32, height as u32)
    }
} 
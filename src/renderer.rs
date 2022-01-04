use crate::display::Display;

pub struct DisplayRenderer;

impl DisplayRenderer {
    pub fn draw(&self, display: &Display, output_frame: &mut [u8]) {
        for (i, pixel) in output_frame.chunks_exact_mut(4).enumerate() {
            let x = (i % Display::WIDTH) as usize;
            let y = (i / Display::WIDTH) as usize;
            let is_on = display.pixels[y][x];
            let color = if is_on {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&color);
        }
    }
}

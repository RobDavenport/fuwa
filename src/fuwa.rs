use glam::*;
use pixels::{Error, Pixels, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;

pub struct Fuwa<W: HasRawWindowHandle> {
    pub width: u32,
    pub height: u32,
    pixels: Pixels<W>,
}

impl<W: HasRawWindowHandle> Fuwa<W> {
    pub fn new(width: u32, height: u32, window: &W) -> Self {
        Self {
            width,
            height,
            pixels: Pixels::new(width, height, SurfaceTexture::new(width, height, &*window))
                .unwrap(),
        }
    }

    pub fn clear(&mut self, color: &[u8; 4]) {
        for (_index, pixel) in self.pixels.get_frame().chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(color);
        }
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.pixels.render()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize(width, height);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        let index = self.pos_to_index(x, y);
        if let Some(pixel) = self.pixels.get_frame().get_mut(index..index + 4) {
            pixel.copy_from_slice(color);
        }
    }
    
    fn pos_to_index(&self, x: u32, y: u32) -> usize {
        4 * (x + (y * self.width)) as usize
    }

    pub fn draw_triangle(
        &mut self,
        points: &[u32; 6],
        color: &[u8; 4],
    ) {
        //TODO Write This
    }

    pub fn draw_line(
        &mut self,
        mut x_start: u32,
        mut y_start: u32,
        mut x_end: u32,
        mut y_end: u32,
        color: &[u8; 4],
    ) {
        use std::mem::swap;

        let steep = if i32::abs(x_start as i32 - x_end as i32) < i32::abs(y_start as i32 - y_end as i32) {
            swap(&mut x_start, &mut y_start);
            swap(&mut x_end, &mut y_end);
            true
        } else { false };

        if x_start > x_end {
            swap(&mut x_start, &mut x_end);
            swap(&mut y_start, &mut y_end);
        }

        for x in x_start..=x_end {
            let slope = (x - x_start) as f32 / (x_end - x_start) as f32;
            let y = (y_start as f32 * (1. - slope) + y_end as f32 * slope) as u32;
            if steep {
                self.set_pixel(y, x, color);
            } else {
                self.set_pixel(x, y, color);
            }
        }
    }
}

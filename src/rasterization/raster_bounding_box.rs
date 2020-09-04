use super::Triangle;
use crate::Fuwa;
use glam::*;
use raw_window_handle::HasRawWindowHandle;

pub struct RasterBoundingBox([f32; 4]);

impl RasterBoundingBox {
    pub fn min_x(&self) -> f32 {
        self.0[0]
    }

    pub fn min_y(&self) -> f32 {
        self.0[1]
    }

    pub fn max_x(&self) -> f32 {
        self.0[2]
    }

    pub fn max_y(&self) -> f32 {
        self.0[3]
    }

    pub fn prepare(self) -> [u32; 4] {
        [
            self.0[0] as u32,
            self.0[1] as u32,
            self.0[2] as u32,
            self.0[3] as u32,
        ]
    }
}

impl<W: HasRawWindowHandle> Fuwa<W> {
    pub fn calculate_raster_bb(&self, triangle: &Triangle) -> RasterBoundingBox {
        let zero = Vec3A::zero();
        let width_vec = Vec3A::splat(self.width as f32);
        let height_vec = Vec3A::splat(self.height as f32);
        let x_vec = vec3a(
            triangle.points[0].x(),
            triangle.points[1].x(),
            triangle.points[2].x(),
        );
        let y_vec = vec3a(
            triangle.points[0].y(),
            triangle.points[1].y(),
            triangle.points[2].y(),
        );

        let mut min_x = x_vec.max(zero).min_element();
        let mut max_x = x_vec.min(width_vec).max_element();

        let mut min_y = y_vec.max(zero).min_element();
        let mut max_y = y_vec.min(height_vec).max_element();

        min_x = min_x.floor();
        min_y = min_y.floor();
        max_x = max_x.ceil();
        max_y = max_y.ceil();

        RasterBoundingBox([min_x, min_y, max_x, max_y])
    }
}
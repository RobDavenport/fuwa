use crate::Fuwa;
use glam::*;
use raw_window_handle::HasRawWindowHandle;

pub struct Triangle {
    pub(crate) points: [Vec3A; 3],
}

impl Triangle {
    pub fn from_data(vertices: &[Vec3A], indices: &[usize]) -> Self {
        Self {
            points: [
                vertices[indices[0]],
                vertices[indices[1]],
                vertices[indices[2]],
            ],
        }
    }

    pub fn from_points(points: [Vec3A; 3]) -> Self {
        Self { points }
    }

    pub fn is_backfacing(&self) -> bool {
        is_backfacing_points(&self.points)
    }

    pub fn transform_screen_space_orthographic<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_orthographic(&mut self.points[0]);
        fuwa.transform_screen_space_orthographic(&mut self.points[1]);
        fuwa.transform_screen_space_orthographic(&mut self.points[2]);
    }

    pub fn transform_screen_space_perspective<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_perspective(&mut self.points[0]);
        fuwa.transform_screen_space_perspective(&mut self.points[1]);
        fuwa.transform_screen_space_perspective(&mut self.points[2]);
    }
}

pub fn is_backfacing_points(points: &[Vec3A; 3]) -> bool {
    let e1 = points[1] - points[0];
    let e2 = points[2] - points[0];
    e1.cross(e2).dot(points[0]).is_sign_negative()
}

use glam::*;

pub struct Triangle {
    pub(crate) points: [Vec3A; 3],
}

impl Triangle {
    pub fn from_data(vertices: &[Vec3A], indices: &[u32]) -> Self {
        Self {
            points: [
                vertices[indices[0] as usize],
                vertices[indices[1] as usize],
                vertices[indices[2] as usize],
            ],
        }
    }

    pub fn from_points(points: [Vec3A; 3]) -> Self {
        Self { points }
    }

    pub fn is_backfacing_triangle(&self) -> bool {
        is_backfacing_points(&self.points)
    }
}

pub fn is_backfacing_points(points: &[Vec3A; 3]) -> bool {
    let e1 = points[1] - points[0];
    let e2 = points[2] - points[0];
    e1.cross(e2).dot(points[0]).is_sign_negative()
}

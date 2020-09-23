use crate::{FSInput, FragmentShader, Fuwa};
use glam::*;
use once_cell::sync::OnceCell;
use raw_window_handle::HasRawWindowHandle;

pub struct Triangle<F> {
    pub(crate) points: [Vec3A; 3],
    pub(crate) vs_input: [F; 3],
    interpolate_diffs: OnceCell<[F; 3]>,
    z_diffs: OnceCell<[f32; 3]>,
}

impl<F: FSInput> Triangle<F> {
    pub(crate) fn new(points: [Vec3A; 3], vs_input: [F; 3]) -> Self {
        Self {
            points,
            vs_input,
            interpolate_diffs: OnceCell::new(),
            z_diffs: OnceCell::new(),
        }
    }

    //TODO: Change this to a struct?
    pub(crate) fn get_interpolate_diffs(&self) -> &[F; 3] {
        self.interpolate_diffs
            .get_or_try_init(|| -> Result<[F; 3], ()> {
                let v0 = self.vs_input[0];
                let v1 = self.vs_input[1] - v0;
                let v2 = self.vs_input[2] - v0;
                Ok([v0, v1, v2])
            })
            .unwrap()
    }

    //TODO: Change this to a struct?
    pub(crate) fn get_z_diffs(&self) -> &[f32; 3] {
        self.z_diffs
            .get_or_try_init(|| -> Result<[f32; 3], ()> {
                let z0 = self.points[0].z();
                let z1 = self.points[1].z();
                let z2 = self.points[2].z();
                let zs10 = z1 - z0;
                let zs20 = z2 - z0;

                Ok([z0, zs10, zs20])
            })
            .unwrap()
    }

    pub fn get_points_as_vec3a(&self) -> &[Vec3A; 3] {
        &self.points
    }

    pub fn get_points_as_vec2(&self) -> [Vec2; 3] {
        [
            self.points[0].truncate(),
            self.points[1].truncate(),
            self.points[2].truncate(),
        ]
    }

    pub fn is_backfacing(&self) -> bool {
        is_backfacing_points(&self.points)
    }

    pub fn transform_screen_space_orthographic<
        W: HasRawWindowHandle + Send + Sync,
    >(
        &mut self,
        fuwa: &Fuwa<W>,
    ) {
        fuwa.transform_screen_space_orthographic(&mut self.points[0]);
        fuwa.transform_screen_space_orthographic(&mut self.points[1]);
        fuwa.transform_screen_space_orthographic(&mut self.points[2]);
    }

    pub fn transform_screen_space_perspective<
        W: HasRawWindowHandle + Send + Sync,
    >(
        &mut self,
        fuwa: &Fuwa<W>,
    ) {
        fuwa.transform_screen_space_perspective(&mut self.points[0], &mut self.vs_input[0]);
        fuwa.transform_screen_space_perspective(&mut self.points[1], &mut self.vs_input[1]);
        fuwa.transform_screen_space_perspective(&mut self.points[2], &mut self.vs_input[2]);
    }
}

pub(crate) fn is_backfacing_points(points: &[Vec3A; 3]) -> bool {
    let e1 = points[1] - points[0];
    let e2 = points[2] - points[0];
    e1.cross(e2).dot(points[0]).is_sign_negative()
}

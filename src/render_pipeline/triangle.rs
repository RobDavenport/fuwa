use crate::{Fuwa, VertexData};
use core::slice::Iter;
use glam::*;
use itertools::izip;
use once_cell::sync::OnceCell;
use raw_window_handle::HasRawWindowHandle;

pub struct Triangle<'a> {
    pub(crate) points: [VertexData<'a>; 3],
    pub(crate) position_index: usize,
    interpolate_diffs: OnceCell<[Vec<f32>; 3]>,
    z_diffs: OnceCell<[f32; 3]>,
}

impl<'a> Triangle<'a> {
    pub(crate) fn from_points(
        v0: VertexData<'a>,
        v1: VertexData<'a>,
        v2: VertexData<'a>,
        position_index: usize,
    ) -> Self {
        Self {
            points: [v0, v1, v2],
            interpolate_diffs: OnceCell::new(),
            z_diffs: OnceCell::new(),
            position_index,
        }
    }

    //TODO: Change this to a struct?
    pub(crate) fn get_interpolate_diffs(&self) -> &[Vec<f32>; 3] {
        self.interpolate_diffs
            .get_or_try_init(|| -> Result<[Vec<f32>; 3], ()> {
                let (p0i, p1i, p2i, len) = self.get_vertex_iterators();

                let mut p0s = Vec::with_capacity(len);
                let mut sub10 = Vec::with_capacity(len);
                let mut sub20 = Vec::with_capacity(len);

                for (p0, p1, p2) in izip!(p0i, p1i, p2i) {
                    p0s.push(*p0);
                    sub10.push(*p1 - *p0);
                    sub20.push(*p2 - *p0);
                }

                Ok([p0s, sub10, sub20])
            })
            .unwrap()
    }

    //TODO: Change this to a struct?
    pub(crate) fn get_z_diffs(&self) -> &[f32; 3] {
        self.z_diffs
            .get_or_try_init(|| -> Result<[f32; 3], ()> {
                let z_index = self.position_index + 2;
                let z0 = self.points[0].raw_data[z_index];
                let z1 = self.points[1].raw_data[z_index];
                let z2 = self.points[2].raw_data[z_index];
                let zs10 = z1 - z0;
                let zs20 = z2 - z0;

                Ok([z0, zs10, zs20])
            })
            .unwrap()
    }

    fn get_vertex_iterators(&self) -> (Iter<f32>, Iter<f32>, Iter<f32>, usize) {
        (
            self.points[0].raw_data.iter(),
            self.points[1].raw_data.iter(),
            self.points[2].raw_data.iter(),
            self.points[0].raw_data.len(),
        )
    }

    pub fn get_points_as_vec3a(&self) -> [Vec3A; 3] {
        [
            Vec3A::from_slice_unaligned(
                &self.points[0].raw_data[self.position_index..self.position_index + 3],
            ),
            Vec3A::from_slice_unaligned(
                &self.points[1].raw_data[self.position_index..self.position_index + 3],
            ),
            Vec3A::from_slice_unaligned(
                &self.points[2].raw_data[self.position_index..self.position_index + 3],
            ),
        ]
    }

    pub fn get_points_as_vec2(&self) -> [Vec2; 3] {
        [
            Vec2::from_slice_unaligned(
                &self.points[0].raw_data[self.position_index..self.position_index + 2],
            ),
            Vec2::from_slice_unaligned(
                &self.points[1].raw_data[self.position_index..self.position_index + 2],
            ),
            Vec2::from_slice_unaligned(
                &self.points[2].raw_data[self.position_index..self.position_index + 2],
            ),
        ]
    }

    pub fn is_backfacing(&self) -> bool {
        is_backfacing_points(&self.points, self.position_index)
    }

    pub fn transform_screen_space_orthographic<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_orthographic(
            &mut self.points[0].raw_data[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_orthographic(
            &mut self.points[1].raw_data[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_orthographic(
            &mut self.points[2].raw_data[self.position_index..self.position_index + 3],
        );
    }

    pub fn transform_screen_space_perspective<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_perspective(&mut self.points[0].raw_data, self.position_index);
        fuwa.transform_screen_space_perspective(&mut self.points[1].raw_data, self.position_index);
        fuwa.transform_screen_space_perspective(&mut self.points[2].raw_data, self.position_index);
    }
}

pub(crate) fn is_backfacing_points(points: &[VertexData; 3], position_index: usize) -> bool {
    let positions = [
        points[0].get_position(position_index),
        points[1].get_position(position_index),
        points[2].get_position(position_index),
    ];

    let e1 = positions[1] - positions[0];
    let e2 = positions[2] - positions[0];
    e1.cross(e2).dot(positions[0]).is_sign_negative()
}

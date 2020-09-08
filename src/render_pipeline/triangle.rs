use crate::{Fuwa, VertexData};
use glam::*;
use raw_window_handle::HasRawWindowHandle;

pub struct Triangle<'a> {
    pub(crate) points: [VertexData<'a>; 3],
    pub(crate) position_index: usize,
}

impl<'a> Triangle<'a> {
    // pub fn from_data(vertices: &[VertexData], indices: &[usize]) -> Self {
    //     Self {
    //         points: [
    //             vertices[indices[0]],
    //             vertices[indices[1]],
    //             vertices[indices[2]],
    //         ],
    //     }
    // }

    pub fn from_points(
        v0: VertexData<'a>,
        v1: VertexData<'a>,
        v2: VertexData<'a>,
        position_index: usize,
    ) -> Self {
        Self {
            points: [v0, v1, v2],
            position_index,
        }
    }

    pub fn get_points_as_vec3a(&self) -> [Vec3A; 3] {
        [
            Vec3A::from_slice_unaligned(
                &self.points[0].0[self.position_index..self.position_index + 3],
            ),
            Vec3A::from_slice_unaligned(
                &self.points[1].0[self.position_index..self.position_index + 3],
            ),
            Vec3A::from_slice_unaligned(
                &self.points[2].0[self.position_index..self.position_index + 3],
            ),
        ]
    }

    pub fn get_points_as_vec2(&self) -> [Vec2; 3] {
        [
            Vec2::from_slice_unaligned(
                &self.points[0].0[self.position_index..self.position_index + 2],
            ),
            Vec2::from_slice_unaligned(
                &self.points[1].0[self.position_index..self.position_index + 2],
            ),
            Vec2::from_slice_unaligned(
                &self.points[2].0[self.position_index..self.position_index + 2],
            ),
        ]
    }

    // pub fn from_points(points: [Vec3A; 3]) -> Self {
    //     Self { points }
    // }

    pub fn is_backfacing(&self) -> bool {
        is_backfacing_points(&self.points, self.position_index)
    }

    pub fn transform_screen_space_orthographic<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_orthographic(
            &mut self.points[0].0[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_orthographic(
            &mut self.points[1].0[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_orthographic(
            &mut self.points[2].0[self.position_index..self.position_index + 3],
        );
    }

    pub fn transform_screen_space_perspective<F: HasRawWindowHandle + Send + Sync>(
        &mut self,
        fuwa: &Fuwa<F>,
    ) {
        fuwa.transform_screen_space_perspective(
            &mut self.points[0].0[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_perspective(
            &mut self.points[1].0[self.position_index..self.position_index + 3],
        );
        fuwa.transform_screen_space_perspective(
            &mut self.points[2].0[self.position_index..self.position_index + 3],
        );
    }
}

pub fn is_backfacing_points(points: &[VertexData; 3], position_index: usize) -> bool {
    let positions = [
        points[0].get_position(position_index),
        points[1].get_position(position_index),
        points[2].get_position(position_index),
    ];

    let e1 = positions[1] - positions[0];
    let e2 = positions[2] - positions[0];
    e1.cross(e2).dot(positions[0]).is_sign_negative()
}

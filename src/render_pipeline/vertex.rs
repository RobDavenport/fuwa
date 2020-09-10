use glam::*;

pub(crate) struct VertexData<'a> {
    pub raw_data: &'a mut [f32],
}

impl<'a> VertexData<'a> {
    pub(crate) fn get_position(&self, position_index: usize) -> Vec3A {
        vec3a(
            self.raw_data[position_index],
            self.raw_data[position_index + 1],
            self.raw_data[position_index + 2],
        )
    }

    pub(crate) fn new(raw_data: &'a mut [f32]) -> Self {
        Self { raw_data }
    }
}

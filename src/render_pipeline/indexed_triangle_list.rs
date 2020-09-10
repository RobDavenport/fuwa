use glam::*;

pub struct IndexedVertexList<'a> {
    pub vertex_list: &'a mut [f32],
    pub index_list: &'a [usize],
}

impl<'a> IndexedVertexList<'a> {
    pub(crate) fn clone_vertex_data(&self) -> Vec<f32> {
        self.vertex_list.to_vec()
    }
}

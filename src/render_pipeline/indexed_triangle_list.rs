use glam::*;

pub struct IndexedVertexList<'a> {
    pub vertex_list: &'a mut [f32],
    pub index_list: &'a [usize],
}

impl<'a> IndexedVertexList<'a> {
    pub(crate) fn create_copy(&self, out: &'a mut Vec<f32>) -> Self {
        out.resize(self.vertex_list.len(), 0.);
        out.copy_from_slice(self.vertex_list);
        Self {
            vertex_list: out,
            index_list: self.index_list,
        }
    }
}

use glam::*;

pub struct IndexedTriangleList<'a> {
    pub vertex_list: &'a [Vec3A],
    pub index_list: &'a [usize],
}

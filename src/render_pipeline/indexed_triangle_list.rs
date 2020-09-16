pub struct IndexedVertexList<'a, V: Send + Sync> {
    pub raw_vertex_list: &'a [V],
    pub index_list: &'a [usize],
}

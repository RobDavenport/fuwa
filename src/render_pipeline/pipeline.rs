use super::shaders::FragmentShader;
use super::Triangle;
use super::VertexDescriptor;
use crate::{rasterization::rasterizer, Fuwa, IndexedVertexList, VertexData};
use glam::*;
use itertools::izip;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

pub struct Pipeline {
    pub(crate) rotation: Mat3,
    pub(crate) translation: Vec3A,
    pub(crate) fragment_shader: FragmentShader,
    pub(crate) vertex_descriptor: VertexDescriptor,
    //Vertex Shader?
}
impl Pipeline {
    pub fn new(vertex_descriptor: VertexDescriptor, fragment_shader: FragmentShader) -> Self {
        Self {
            rotation: Mat3::identity(),
            translation: Vec3A::zero(),
            fragment_shader,
            vertex_descriptor,
        }
    }

    pub(crate) fn get_self_ptr(&self) -> PipelinePtr {
        PipelinePtr(self as *const Self)
    }

    pub fn bind_rotation(&mut self, rotation: Mat3) {
        self.rotation = rotation;
    }

    pub fn bind_translation(&mut self, translation: Vec3A) {
        self.translation = translation;
    }

    pub fn draw<W: HasRawWindowHandle + Send + Sync>(
        &self,
        fuwa: &mut Fuwa<W>,
        indexed_list: &IndexedVertexList,
    ) {
        //optick::next_frame();
        let copied_vertex_data = &mut indexed_list.clone_vertex_data();
        self.process_triangle_list(copied_vertex_data);
        self.assemble_triangles(fuwa, copied_vertex_data, &indexed_list.index_list)
    }

    fn process_triangle_list(&self, vertex_list: &mut [f32]) {
        //transform all incoming verts,
        //and prepares them for assembly
        let position = self.vertex_descriptor.position_index;
        vertex_list
            .par_chunks_exact_mut(self.vertex_descriptor.stride)
            .for_each(|chunk| {
                let data = &mut chunk[position..position + 3];
                let result = self.rotation * vec3a(data[0], data[1], data[2]) + self.translation;
                data[0] = result.x();
                data[1] = result.y();
                data[2] = result.z();
            });
    }

    fn assemble_triangles<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        vertex_list: &[f32],
        index_list: &[usize],
    ) {
        //loop through and build triangles,
        //also do backface culling if necessary

        let self_ptr = self.get_self_ptr();
        let fuwa_ptr = fuwa.get_self_ptr();
        let pos_index = self.vertex_descriptor.position_index;
        let stride = self.vertex_descriptor.stride;

        index_list.par_chunks_exact(3).for_each(|indices| unsafe {
            let v0iter = vertex_list[indices[0] * stride..(indices[0] * stride) + stride].iter();
            let v1iter = vertex_list[indices[1] * stride..(indices[1] * stride) + stride].iter();
            let v2iter = vertex_list[indices[2] * stride..(indices[2] * stride) + stride].iter();

            let mut v0_copy = Vec::with_capacity(stride);
            let mut v1_copy = Vec::with_capacity(stride);
            let mut v2_copy = Vec::with_capacity(stride);

            for (v0, v1, v2) in izip!(v0iter, v1iter, v2iter) {
                v0_copy.push(*v0);
                v1_copy.push(*v1);
                v2_copy.push(*v2);
            }

            let mut triangle = Triangle::from_points(
                VertexData::new(&mut v0_copy),
                VertexData::new(&mut v1_copy),
                VertexData::new(&mut v2_copy),
                pos_index,
            );

            if !triangle.is_backfacing() {
                (*self_ptr.0).process_triangle(&mut *fuwa_ptr.0, &mut triangle)
            }
        });
    }

    fn process_triangle<'fs, W: HasRawWindowHandle + Sync + Send>(
        &'fs self,
        fuwa: &mut Fuwa<'fs, W>,
        triangle: &mut Triangle,
    ) {
        //Do something later
        self.post_process_triangle(fuwa, triangle);
    }

    fn post_process_triangle<'fs, W: HasRawWindowHandle + Sync + Send>(
        &'fs self,
        fuwa: &mut Fuwa<'fs, W>,
        triangle: &mut Triangle,
    ) {
        //Transform triangle to screen space
        triangle.transform_screen_space_perspective(fuwa);

        //Draw the triangle
        rasterizer::triangle(
            fuwa.get_self_ptr(),
            triangle,
            &self.fragment_shader.fragment_shader,
        );
    }
}

#[derive(Copy, Clone)]
pub(crate) struct PipelinePtr(pub(crate) *const Pipeline);

unsafe impl Send for PipelinePtr {}
unsafe impl Sync for PipelinePtr {}

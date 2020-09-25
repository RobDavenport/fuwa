use super::Triangle;
use crate::{
    rasterization::{rasterizer, SlabPtr},
    Fuwa, IndexedVertexList,
};
use crate::{FSInput, VSInput, VertexShader};
use glam::*;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

const VERTICES_PER_VERTEX_SHADER_JOB: usize = 8;
const TRIANGLES_PER_RASTER_JOB: usize = 1;

// pub struct Pipeline<V: VSInput, F: FSInput> {
//     pub(crate) fragment_shader: FragmentShader<F>,
//     pub(crate) vertex_shader: Box<dyn VertexShader<V, F>>,
// }
//impl<V: VSInput, F: FSInput> Pipeline<V, F> {
// pub fn new(
//     fragment_shader: FragmentShader<F>,
//     vertex_shader: Box<dyn VertexShader<V, F>>,
// ) -> Self {
//     Self {
//         vertex_shader,
//         fragment_shader,
//     }
// }

// pub(crate) fn get_self_ptr(&self) -> PipelinePtr<V, F> {
//     PipelinePtr(self as *const Self)
// }

pub fn draw<V: VSInput, F: FSInput, W: HasRawWindowHandle + Send + Sync>(
    // &'fs self,
    fuwa: &mut Fuwa<W>,
    vertex_shader: &impl VertexShader<V, F>,
    fs_index: usize,
    indexed_list: &IndexedVertexList<V>,
) {
    //optick::next_frame();
    let vs_output = indexed_list
        .raw_vertex_list
        .par_chunks(VERTICES_PER_VERTEX_SHADER_JOB)
        .flat_map(|vertices| {
            vertices
                .into_iter()
                .map(|v| vertex_shader.vertex_shader_fn(v))
                .collect::<Vec<(Vec3A, F)>>()
        })
        .collect::<Vec<(Vec3A, F)>>();

    assemble_triangles(fuwa, vs_output, fs_index, &indexed_list.index_list)
}

// fn run_vertex_shader(vertex_list: &[V]) -> Vec<(Vec3A, F)> {
//     //transform all incoming verts,
//     //and prepares them for assembly
//     vertex_list
//         .into_par_iter()
//         .map(|vertex| self.vertex_shader.vertex_shader_fn(vertex))
//         .collect::<Vec<(Vec3A, F)>>()
// }

fn assemble_triangles<F: FSInput, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<W>,
    vs_output: Vec<(Vec3A, F)>,
    fs_index: usize,
    index_list: &[usize],
) {
    //loop through and build triangles,
    //also do backface culling if necessary

    //let self_ptr = self.get_self_ptr();
    let fuwa_ptr = fuwa.get_self_ptr();
    let slab_ptr = SlabPtr(fuwa.fragment_slab_map.get_mut_slab::<F>());

    index_list
        .par_chunks(3 * TRIANGLES_PER_RASTER_JOB)
        .for_each(|job_indices| unsafe {
            job_indices.chunks_exact(3).for_each(|tri_indices| {
                let idx0 = tri_indices[0];
                let idx1 = tri_indices[1];
                let idx2 = tri_indices[2];

                let mut triangle = Triangle::new(
                    [vs_output[idx0].0, vs_output[idx1].0, vs_output[idx2].0],
                    [vs_output[idx0].1, vs_output[idx1].1, vs_output[idx2].1],
                );

                if !triangle.is_backfacing() {
                    process_triangle(&mut *fuwa_ptr.0, &mut triangle, fs_index, slab_ptr)
                }
            })
        });
}

fn process_triangle<F: FSInput, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<W>,
    triangle: &mut Triangle<F>,
    fs_index: usize,
    slab_ptr: SlabPtr<F>,
) {
    //Do something later
    post_process_triangle(fuwa, triangle, fs_index, slab_ptr);
}

fn post_process_triangle<F: FSInput, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<W>,
    triangle: &mut Triangle<F>,
    fs_index: usize,
    slab_ptr: SlabPtr<F>,
) {
    //Transform triangle to screen space
    triangle.transform_screen_space_perspective(fuwa);

    //Draw the triangle
    rasterizer::triangle(fuwa.get_self_ptr(), triangle, fs_index, slab_ptr);
}
//}

// #[derive(Copy, Clone)]
// pub(crate) struct PipelinePtr<V: VSInput, F: FSInput>(pub(crate) *const Pipeline<V, F>);

// unsafe impl<V: VSInput, F: FSInput> Send for PipelinePtr<V, F> {}
// unsafe impl<V: VSInput, F: FSInput> Sync for PipelinePtr<V, F> {}

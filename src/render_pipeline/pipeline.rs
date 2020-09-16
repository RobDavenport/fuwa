use super::shaders::FragmentShader;
use super::Triangle;
use crate::{rasterization::rasterizer, Fuwa, IndexedVertexList};
use crate::{FSInput, VSInput, VertexShader};
use glam::*;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

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

pub fn draw<V: VSInput, F: FSInput, S: FragmentShader<F>, W: HasRawWindowHandle + Send + Sync>(
    // &'fs self,
    fuwa: &mut Fuwa<F, S, W>,
    vertex_shader: &impl VertexShader<V, F>,
    fragment_shader: &S,
    indexed_list: &IndexedVertexList<V>,
) {
    //optick::next_frame();
    let vs_output = indexed_list
        .raw_vertex_list
        .into_par_iter()
        .map(|vertex| vertex_shader.vertex_shader_fn(vertex))
        .collect::<Vec<(Vec3A, F)>>();

    assemble_triangles(fuwa, vs_output, fragment_shader, &indexed_list.index_list)
}

// fn run_vertex_shader(vertex_list: &[V]) -> Vec<(Vec3A, F)> {
//     //transform all incoming verts,
//     //and prepares them for assembly
//     vertex_list
//         .into_par_iter()
//         .map(|vertex| self.vertex_shader.vertex_shader_fn(vertex))
//         .collect::<Vec<(Vec3A, F)>>()
// }

fn assemble_triangles<F: FSInput, S: FragmentShader<F>, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<F, S, W>,
    vs_output: Vec<(Vec3A, F)>,
    fragment_shader: &S,
    index_list: &[usize],
) {
    //loop through and build triangles,
    //also do backface culling if necessary

    //let self_ptr = self.get_self_ptr();
    let fuwa_ptr = fuwa.get_self_ptr();

    index_list.par_chunks_exact(3).for_each(|indices| unsafe {
        let idx0 = indices[0];
        let idx1 = indices[1];
        let idx2 = indices[2];

        let mut triangle = Triangle::new(
            [vs_output[idx0].0, vs_output[idx1].0, vs_output[idx2].0],
            [vs_output[idx0].1, vs_output[idx1].1, vs_output[idx2].1],
        );

        if !triangle.is_backfacing() {
            //(*self_ptr.0).
            process_triangle(&mut *fuwa_ptr.0, &mut triangle, fragment_shader)
        }
    });
}

fn process_triangle<F: FSInput, S: FragmentShader<F>, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<F, S, W>,
    triangle: &mut Triangle<F>,
    fragment_shader: &S,
) {
    //Do something later
    post_process_triangle(fuwa, triangle, fragment_shader);
}

fn post_process_triangle<F: FSInput, S: FragmentShader<F>, W: HasRawWindowHandle + Sync + Send>(
    //&'fs self,
    fuwa: &mut Fuwa<F, S, W>,
    triangle: &mut Triangle<F>,
    fragment_shader: &S,
) {
    //Transform triangle to screen space
    triangle.transform_screen_space_perspective(fuwa);

    //Draw the triangle
    rasterizer::triangle(fuwa.get_self_ptr(), triangle, fragment_shader);
}
//}

// #[derive(Copy, Clone)]
// pub(crate) struct PipelinePtr<V: VSInput, F: FSInput>(pub(crate) *const Pipeline<V, F>);

// unsafe impl<V: VSInput, F: FSInput> Send for PipelinePtr<V, F> {}
// unsafe impl<V: VSInput, F: FSInput> Sync for PipelinePtr<V, F> {}

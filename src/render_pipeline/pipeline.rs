use crate::{Edge, Fuwa, FuwaPtr};
use glam::*;
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

use super::rasterization as raster;
use super::shaders::FragmentShader;
use super::IndexedTriangleList;
use super::Triangle;

lazy_static! {
    static ref RENDER_MASK: Vec4 = Vec4::splat(0.);
}

pub struct Pipeline {
    pub(crate) rotation: Mat3,
    pub(crate) translation: Vec3A,
    pub(crate) fragment_shader: FragmentShader,
    //Vertex Shader?
}
impl Pipeline {
    pub fn new(fragment_shader: FragmentShader) -> Self {
        Self {
            rotation: Mat3::identity(),
            translation: Vec3A::zero(),
            fragment_shader,
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
        triangle_list: &IndexedTriangleList,
    ) {
        let processed_triangles = self.process_triangle_list(&triangle_list.vertex_list);
        self.assemble_triangles(fuwa, &processed_triangles, &triangle_list.index_list)
    }

    fn process_triangle_list(&self, vertex_list: &[Vec3A]) -> Vec<Vec3A> {
        //transform all incoming verts,
        //and prepares them for assembly
        vertex_list
            .par_iter()
            .map(|vertex| self.rotation * *vertex + self.translation)
            .collect::<Vec<Vec3A>>()
    }

    fn assemble_triangles<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        vertex_list: &[Vec3A],
        index_list: &[usize],
    ) {
        //loop through and build triangles,
        //also do backface culling if necessary

        let self_ptr = self.get_self_ptr();
        let fuwa_ptr = fuwa.get_self_ptr();

        index_list.par_chunks_exact(3).for_each(|indices| {
            let mut triangle = Triangle::from_data(vertex_list, indices);

            if !triangle.is_backfacing() {
                unsafe { (*self_ptr.0).process_triangle(&mut *fuwa_ptr.0, &mut triangle) }
            }
        });
    }

    fn process_triangle<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        triangle: &mut Triangle,
    ) {
        //Do something later

        self.post_process_triangle(fuwa, triangle);
    }

    fn post_process_triangle<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        triangle: &mut Triangle,
    ) {
        //Transform triangle to screen space
        triangle.transform_screen_space_perspective(fuwa);

        //Draw the triangle
        //self.draw_triangle_parallel(fuwa, triangle);
        self.draw_triangle_fast(fuwa, triangle);
    }

    // pub fn draw_triangle_parallel<W: HasRawWindowHandle + Send + Sync>(
    //     &self,
    //     fuwa: &mut Fuwa<W>,
    //     triangle: &Triangle
    // ) {
    //     let bb = fuwa.calculate_raster_bb(triangle);

    //     let fuwa_ptr = fuwa.get_self_ptr();

    //     let p = Vec3A::new(bb.min_x(), bb.min_y(), 0.0);
    //     let (e12, w0_row) = Edge::init(&triangle.points[1], &triangle.points[2], &p);
    //     let (e20, w1_row) = Edge::init(&triangle.points[2], &triangle.points[0], &p);
    //     let (e01, w2_row) = Edge::init(&triangle.points[0], &triangle.points[1], &p);

    //     let bb = bb.prepare();

    //     unsafe {
    //         //(0..self.raster_par_count)
    //         //.into_par_iter()
    //         //.for_each(|thread_offset| {
    //         self.rasterize_triangle(
    //             &fuwa_ptr,
    //             triangle,
    //             [e12, e20, e01],
    //             [w0_row, w1_row, w2_row],
    //             bb,
    //             //(0, 1),
    //             //(thread_offset, self.raster_par_count),
    //             //color,
    //         );
    //         //});
    //     }
    // }

    // unsafe fn rasterize_triangle<W: HasRawWindowHandle + Send + Sync>(
    //     &self,
    //     ptr: &FuwaPtr<W>,
    //     triangle: &Triangle,
    //     [e12, e20, e01]: [Edge; 3],
    //     [mut w0_row, mut w1_row, mut w2_row]: [Vec4; 3],
    //     [start_x, start_y, end_x, end_y]: [u32; 4],
    //     //(par_offset, par_count): (usize, usize),
    // ) {
    //     // let step_x_offset = [
    //     //     e12.one_step_x * par_offset as f32,
    //     //     e20.one_step_x * par_offset as f32,
    //     //     e01.one_step_x * par_offset as f32,
    //     // ];

    //     // let step_x_count = [
    //     //     e12.one_step_x * par_count as f32,
    //     //     e20.one_step_x * par_count as f32,
    //     //     e01.one_step_x * par_count as f32,
    //     // ];

    //     (start_y..end_y).step_by(Edge::STEP_Y).for_each(|y| {
    //         // let mut w0 = w0_row + step_x_offset[0];
    //         // let mut w1 = w1_row + step_x_offset[1];
    //         // let mut w2 = w2_row + step_x_offset[2];

    //         let mut w0 = w0_row;
    //         let mut w1 = w1_row;
    //         let mut w2 = w2_row;

    //         (start_x..end_x)
    //             .step_by(Edge::STEP_X)
    //             //.skip(par_offset as usize * Edge::STEP_X)
    //             //.step_by(Edge::STEP_X as usize * par_count)
    //             .for_each(|x| {
    //                 let pixel_mask =
    //                     w0.cmple(*RENDER_MASK) & w1.cmple(*RENDER_MASK) & w2.cmple(*RENDER_MASK);

    //                 if pixel_mask.any() {
    //                     let weight_sum = w0 + w1 + w2;
    //                     let l1 = w1 / weight_sum;
    //                     let l2 = w2 / weight_sum;

    //                     let pz = Vec4::splat(triangle.points[0].z())
    //                         + (l1 * (triangle.points[1].z() - triangle.points[0].z()))
    //                         + (l2 * (triangle.points[2].z() - triangle.points[0].z()));

    //                     if let Some(depth_pass) = (*ptr.0).try_set_depth_simd(x, y, pz, pixel_mask) {
    //                         (*ptr.0).set_pixels_unchecked(x as u32, y, depth_pass, &(self.fragment_shader.fragment_shader)());
    //                     }
    //                 }

    //                 // w0 += step_x_count[0];
    //                 // w1 += step_x_count[1];
    //                 // w2 += step_x_count[2];

    //                 w0 += e12.one_step_x;
    //                 w1 += e20.one_step_x;
    //                 w2 += e01.one_step_x;
    //             });

    //         w0_row += e12.one_step_y;
    //         w1_row += e20.one_step_y;
    //         w2_row += e01.one_step_y;
    //     });
    // }

    pub fn draw_triangle_fast<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        triangle: &Triangle,
    ) {
        let bb = fuwa.calculate_raster_bb(triangle);

        let a01 = triangle.points[0].y() - triangle.points[1].y();
        let a12 = triangle.points[1].y() - triangle.points[2].y();
        let a20 = triangle.points[2].y() - triangle.points[0].y();

        let b01 = triangle.points[1].x() - triangle.points[0].x();
        let b12 = triangle.points[2].x() - triangle.points[1].x();
        let b20 = triangle.points[0].x() - triangle.points[2].x();

        let p = Vec3A::new(bb.min_x(), bb.min_y(), 0.0);

        let mut w0_row = orient_2d(&triangle.points[1], &triangle.points[2], &p);
        let mut w1_row = orient_2d(&triangle.points[2], &triangle.points[0], &p);
        let mut w2_row = orient_2d(&triangle.points[0], &triangle.points[1], &p);

        let fuwa_ptr = fuwa.get_self_ptr();
        unsafe {
            (bb.min_y() as u32..bb.max_y() as u32).for_each(|y| {
                let mut w0 = w0_row;
                let mut w1 = w1_row;
                let mut w2 = w2_row;

                (bb.min_x() as u32..bb.max_x() as u32).for_each(|x| {
                    if w0.is_sign_negative() && w1.is_sign_negative() && w2.is_sign_negative() {
                        let weight_sum = w0 + w1 + w2;
                        let l0 = w1 / weight_sum;
                        let l1 = w1 / weight_sum;
                        let l2 = w2 / weight_sum;

                        let px = triangle.points[0].x()
                            + (l1 * (triangle.points[1].x() - triangle.points[0].x()))
                            + (l2 * (triangle.points[2].x() - triangle.points[0].x()));

                        let py = triangle.points[0].y()
                            + (l1 * (triangle.points[1].y() - triangle.points[0].y()))
                            + (l2 * (triangle.points[2].y() - triangle.points[0].y()));

                        let pz = triangle.points[0].z()
                            + (l1 * (triangle.points[1].z() - triangle.points[0].z()))
                            + (l2 * (triangle.points[2].z() - triangle.points[0].z()));

                        let input = [px, py, pz];

                        if fuwa.try_set_depth(x, y, pz) {
                            (*fuwa_ptr.0).set_pixel_unchecked(
                                x,
                                y,
                                &(self.fragment_shader.fragment_shader)(&input),
                            );
                        }
                    }

                    w0 += a12;
                    w1 += a20;
                    w2 += a01;
                });

                w0_row += b12;
                w1_row += b20;
                w2_row += b01;
            });
        }
    }
}

fn orient_2d(a: &Vec3A, b: &Vec3A, point: &Vec3A) -> f32 {
    (b.x() - a.x()) * (point.y() - a.y()) - (b.y() - a.y()) * (point.x() - a.x())
}

unsafe fn vec4_from_pixel_ptr(ptr: *const f32) -> Vec4 {
    use std::ptr::slice_from_raw_parts;
    let data = slice_from_raw_parts(ptr, 4);
    Vec4::from_slice_unaligned(&*data)
}

#[derive(Copy, Clone)]
pub(crate) struct PipelinePtr(pub(crate) *const Pipeline);

unsafe impl Send for PipelinePtr {}
unsafe impl Sync for PipelinePtr {}

use crate::{Edge, Fuwa, FuwaPtr, IndexedVertexList, VertexData};
use glam::*;
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

use super::rasterization as raster;
use super::shaders::FragmentShader;
use super::Triangle;
use super::VertexDescriptor;

lazy_static! {
    static ref RENDER_MASK: Vec4 = Vec4::splat(0.);
}

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
        let mut copied_vertex_data = Vec::<f32>::with_capacity(indexed_list.vertex_list.len());
        let mut render_copy = indexed_list.create_copy(&mut copied_vertex_data);
        self.process_triangle_list(&mut render_copy);
        self.assemble_triangles(fuwa, &mut render_copy.vertex_list, &indexed_list.index_list)
    }

    fn process_triangle_list(&self, vertex_list: &mut IndexedVertexList) {
        //transform all incoming verts,
        //and prepares them for assembly
        let position = self.vertex_descriptor.position_index;
        vertex_list
            .vertex_list
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
        vertex_list: &mut [f32],
        index_list: &[usize],
    ) {
        //loop through and build triangles,
        //also do backface culling if necessary

        let self_ptr = self.get_self_ptr();
        let fuwa_ptr = fuwa.get_self_ptr();
        let pos_index = self.vertex_descriptor.position_index;
        let stride = self.vertex_descriptor.stride;

        index_list.par_chunks_exact(3).for_each(|indices| unsafe {
            //let mut triangle = Triangle::from_data(vertex_list, indices);
            // let v0 = vertex_list[indices[0] * stride..(indices[0] * stride) + stride].as_ptr() as *mut f32;
            // let v1 = vertex_list[indices[1] * stride..(indices[1] * stride) + stride].as_ptr() as *mut f32;
            // let v2 = vertex_list[indices[2] * stride..(indices[2] * stride) + stride].as_ptr() as *mut f32;

            let v0 = &vertex_list[indices[0] * stride..(indices[0] * stride) + stride];
            let v1 = &vertex_list[indices[1] * stride..(indices[1] * stride) + stride];
            let v2 = &vertex_list[indices[2] * stride..(indices[2] * stride) + stride];

            let mut v0_copy = v0.iter().map(|x| *x).collect::<Vec<f32>>();
            let mut v1_copy = v1.iter().map(|x| *x).collect::<Vec<f32>>();
            let mut v2_copy = v2.iter().map(|x| *x).collect::<Vec<f32>>();

            // let r0 = (indices[0] * stride..(indices[0] * stride) + stride);
            // let r1 = (indices[1] * stride..(indices[1] * stride) + stride);
            // let r2 = (indices[2] * stride..(indices[2] * stride) + stride);

            // // println!("r0: {:?}, r1: {:?}, r2: {:?}", &r0, &r1, &r2);

            // let v0 = vertex_list[indices[0] * stride..(indices[0] * stride) + stride].as_ptr() as *mut f32;
            // let v1 = vertex_list[indices[1] * stride..(indices[1] * stride) + stride].as_ptr() as *mut f32;
            // let v2 = vertex_list[indices[2] * stride..(indices[2] * stride) + stride].as_ptr() as *mut f32;

            let mut triangle = Triangle::from_points(
                VertexData(&mut v0_copy),
                VertexData(&mut v1_copy),
                VertexData(&mut v2_copy),
                pos_index,
            );

            // let mut triangle = Triangle::from_points(
            //     VertexData(from_raw_parts_mut(v0, stride)),
            //     VertexData(from_raw_parts_mut(v1, stride)),
            //     VertexData(from_raw_parts_mut(v2, stride)),
            //     pos_index
            // );

            if !triangle.is_backfacing() {
                (*self_ptr.0).process_triangle(&mut *fuwa_ptr.0, &mut triangle)
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
        self.draw_triangle_fast(fuwa, triangle);
        //self.draw_triangle_parallel(fuwa, triangle);
    }

    pub fn draw_triangle_parallel<W: HasRawWindowHandle + Send + Sync>(
        &self,
        fuwa: &mut Fuwa<W>,
        triangle: &Triangle,
    ) {
        optick::event!();
        let points = triangle.get_points_as_vec2();
        let bb = fuwa.calculate_raster_bb(&points);
        let fuwa_ptr = fuwa.get_self_ptr();

        let origin = Vec2::new(bb.min_x(), bb.min_y());
        let (e12, w0_row) = Edge::init(&points[1], &points[2], &origin);
        let (e20, w1_row) = Edge::init(&points[2], &points[0], &origin);
        let (e01, w2_row) = Edge::init(&points[0], &points[1], &origin);

        let bb = bb.prepare();

        unsafe {
            //(0..self.raster_par_count)
            //.into_par_iter()
            //.for_each(|thread_offset| {
            self.rasterize_triangle(
                &fuwa_ptr,
                triangle,
                [e12, e20, e01],
                [w0_row, w1_row, w2_row],
                bb,
                //(0, 1),
                //(thread_offset, self.raster_par_count),
                //color,
            );
            //});
        }
    }

    unsafe fn rasterize_triangle<W: HasRawWindowHandle + Send + Sync>(
        &self,
        ptr: &FuwaPtr<W>,
        triangle: &Triangle,
        [e12, e20, e01]: [Edge; 3],
        [mut w0_row, mut w1_row, mut w2_row]: [Vec4; 3],
        [start_x, start_y, end_x, end_y]: [u32; 4],
        //(par_offset, par_count): (usize, usize),
    ) {
        optick::event!();
        // let step_x_offset = [
        //     e12.one_step_x * par_offset as f32,
        //     e20.one_step_x * par_offset as f32,
        //     e01.one_step_x * par_offset as f32,
        // ];

        // let step_x_count = [
        //     e12.one_step_x * par_count as f32,
        //     e20.one_step_x * par_count as f32,
        //     e01.one_step_x * par_count as f32,
        // ];

        (start_y..end_y).step_by(Edge::STEP_Y).for_each(|y| {
            // let mut w0 = w0_row + step_x_offset[0];
            // let mut w1 = w1_row + step_x_offset[1];
            // let mut w2 = w2_row + step_x_offset[2];

            let mut w0 = w0_row;
            let mut w1 = w1_row;
            let mut w2 = w2_row;

            (start_x..end_x)
                .step_by(Edge::STEP_X)
                //.skip(par_offset as usize * Edge::STEP_X)
                //.step_by(Edge::STEP_X as usize * par_count)
                .for_each(|x| {
                    let pixel_mask =
                        w0.cmple(*RENDER_MASK) & w1.cmple(*RENDER_MASK) & w2.cmple(*RENDER_MASK);

                    if pixel_mask.any() {
                        let weight_sum = w0 + w1 + w2;
                        let l1 = w1 / weight_sum;
                        let l2 = w2 / weight_sum;

                        let position = self.vertex_descriptor.position_index;

                        let pz = Vec4::splat(triangle.points[0].0[position + 2])
                            + (l1
                                * (triangle.points[1].0[position + 2]
                                    - triangle.points[0].0[position + 2]))
                            + (l2
                                * (triangle.points[2].0[position + 2]
                                    - triangle.points[0].0[position + 2]));

                        if let Some(depth_pass) = (*ptr.0).try_set_depth_simd(x, y, pz, pixel_mask)
                        {
                            let len = triangle.points[0].0.len();

                            let mut interp = [
                                Vec::with_capacity(len),
                                Vec::with_capacity(len),
                                Vec::with_capacity(len),
                                Vec::with_capacity(len),
                            ];

                            triangle.points[0]
                                .0
                                .iter()
                                .enumerate()
                                .for_each(|(idx, val)| {
                                    let result = Vec4::splat(*val)
                                        + (l1
                                            * (triangle.points[1].0[idx]
                                                - triangle.points[0].0[idx]))
                                        + (l2
                                            * (triangle.points[2].0[idx]
                                                - triangle.points[0].0[idx]));

                                    interp[0].push(result[0]);
                                    interp[1].push(result[1]);
                                    interp[2].push(result[2]);
                                    interp[3].push(result[3]);
                                });

                            (*ptr.0).set_pixels_unchecked(
                                x as u32,
                                y,
                                depth_pass,
                                self.fragment_shader.fragment_shader,
                                interp,
                            );
                        }
                    }

                    // w0 += step_x_count[0];
                    // w1 += step_x_count[1];
                    // w2 += step_x_count[2];

                    w0 += e12.one_step_x;
                    w1 += e20.one_step_x;
                    w2 += e01.one_step_x;
                });

            w0_row += e12.one_step_y;
            w1_row += e20.one_step_y;
            w2_row += e01.one_step_y;
        });
    }

    pub fn draw_triangle_fast<W: HasRawWindowHandle + Sync + Send>(
        &self,
        fuwa: &mut Fuwa<W>,
        triangle: &Triangle,
    ) {
        let points = triangle.get_points_as_vec3a();
        let points2d = triangle.get_points_as_vec2();
        let bb = fuwa.calculate_raster_bb(&points2d);

        let a01 = points[0].y() - points[1].y();
        let a12 = points[1].y() - points[2].y();
        let a20 = points[2].y() - points[0].y();

        let b01 = points[1].x() - points[0].x();
        let b12 = points[2].x() - points[1].x();
        let b20 = points[0].x() - points[2].x();

        let p = Vec2::new(bb.min_x(), bb.min_y());

        let mut w0_row = orient_2d(&points2d[1], &points2d[2], &p);
        let mut w1_row = orient_2d(&points2d[2], &points2d[0], &p);
        let mut w2_row = orient_2d(&points2d[0], &points2d[1], &p);

        let fuwa_ptr = fuwa.get_self_ptr();
        unsafe {
            (bb.min_y() as u32..bb.max_y() as u32).for_each(|y| {
                let mut w0 = w0_row;
                let mut w1 = w1_row;
                let mut w2 = w2_row;

                (bb.min_x() as u32..bb.max_x() as u32).for_each(|x| {
                    if w0.is_sign_negative() && w1.is_sign_negative() && w2.is_sign_negative() {
                        let weight_sum = w0 + w1 + w2;
                        let l1 = w1 / weight_sum;
                        let l2 = w2 / weight_sum;

                        let pz = points[0].z()
                            + (l1 * (points[1].z() - points[0].z()))
                            + (l2 * (points[2].z() - points[0].z()));

                        if fuwa.try_set_depth(x, y, pz) {
                            let interp = triangle.points[0]
                                .0
                                .iter()
                                .enumerate()
                                .map(|(idx, val)| {
                                    val + (l1
                                        * (triangle.points[1].0[idx] - triangle.points[0].0[idx]))
                                        + (l2
                                            * (triangle.points[2].0[idx]
                                                - triangle.points[0].0[idx]))
                                })
                                .collect::<Vec<_>>();

                            (*fuwa_ptr.0).set_pixel_unchecked(
                                x,
                                y,
                                &(self.fragment_shader.fragment_shader)(&interp),
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

fn orient_2d(a: &Vec2, b: &Vec2, point: &Vec2) -> f32 {
    (b.x() - a.x()) * (point.y() - a.y()) - (b.y() - a.y()) * (point.x() - a.x())
}

#[derive(Copy, Clone)]
pub(crate) struct PipelinePtr(pub(crate) *const Pipeline);

unsafe impl Send for PipelinePtr {}
unsafe impl Sync for PipelinePtr {}

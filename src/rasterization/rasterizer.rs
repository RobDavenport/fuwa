use super::*;
use crate::{Fuwa, FuwaPtr};
use glam::*;
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;

lazy_static! {
    static ref RENDER_MASK: Vec4 = Vec4::splat(0.);
}

impl<W: HasRawWindowHandle + Send + Sync> Fuwa<W> {
    //TODO: Try to write a "Try_Rasterize" func
    //Which uses a function pointer or closure on success
    //That way, we can have a single rasterize check (or more for single/parallel/simd)
    //But then pass the actual shading logic as a parameter
    //This will make stuff like Flat Shaded, VertexColors, Textures better!

    pub fn draw_triangle_fast(&mut self, triangle: &Triangle, color: &[u8; 4]) {
        let bb = self.calculate_raster_bb(triangle);

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

        let self_ptr = self.get_self_ptr();
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

                        let pz = triangle.points[0].z()
                            + (l1 * (triangle.points[1].z() - triangle.points[0].z()))
                            + (l2 * (triangle.points[2].z() - triangle.points[0].z()));

                        if self.try_set_depth(x, y, pz) {
                            (*self_ptr.0).set_pixel_unchecked(x, y, color);
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

    pub fn draw_triangle_parallel(&mut self, triangle: &Triangle, color: &[u8; 4]) {
        let bb = self.calculate_raster_bb(triangle);

        let self_ptr = self.get_self_ptr();

        let p = Vec3A::new(bb.min_x(), bb.min_y(), 0.0);
        let (e12, w0_row) = Edge::init(&triangle.points[1], &triangle.points[2], &p);
        let (e20, w1_row) = Edge::init(&triangle.points[2], &triangle.points[0], &p);
        let (e01, w2_row) = Edge::init(&triangle.points[0], &triangle.points[1], &p);

        let bb = bb.prepare();

        unsafe {
            //(0..self.raster_par_count)
            //.into_par_iter()
            //.for_each(|thread_offset| {
            Self::rasterize_triangle(
                &self_ptr,
                triangle,
                [e12, e20, e01],
                [w0_row, w1_row, w2_row],
                bb,
                (0, 1),
                //(thread_offset, self.raster_par_count),
                color,
            );
            //});
        }
    }

    pub(crate) fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        self.set_pixel_by_index(self.pos_to_index(x, y), color)
    }

    pub(crate) fn set_pixels_unchecked(&mut self, x: u32, y: u32, mask: Vec4Mask, color: &[u8; 4]) {
        let index = self.pos_to_index(x, y);

        unsafe {
            let color_float = f32::from_ne_bytes(*color);

            let current_pixels_ptr = self
                .pixels
                .get_frame()
                .get_unchecked_mut(index..index + 4 * Edge::STEP_X)
                .as_mut_ptr();

            let insert = mask.select(
                Vec4::splat(color_float),
                vec4_from_pixel_ptr(current_pixels_ptr as *const f32),
            );

            current_pixels_ptr.copy_from(
                insert.as_ref().as_ptr() as *const u8,
                (16 - (x.saturating_sub(self.width - Edge::STEP_X as u32) << 2)) as usize,
            );
        };
    }

    unsafe fn rasterize_triangle(
        ptr: &FuwaPtr<W>,
        triangle: &Triangle,
        [e12, e20, e01]: [Edge; 3],
        [mut w0_row, mut w1_row, mut w2_row]: [Vec4; 3],
        [start_x, start_y, end_x, end_y]: [u32; 4],
        (par_offset, par_count): (usize, usize),
        color: &[u8; 4],
    ) {
        let step_x_offset = [
            e12.one_step_x * par_offset as f32,
            e20.one_step_x * par_offset as f32,
            e01.one_step_x * par_offset as f32,
        ];

        let step_x_count = [
            e12.one_step_x * par_count as f32,
            e20.one_step_x * par_count as f32,
            e01.one_step_x * par_count as f32,
        ];

        (start_y..end_y)
            .step_by(Edge::STEP_Y as usize)
            .for_each(|y| {
                let mut w0 = w0_row + step_x_offset[0];
                let mut w1 = w1_row + step_x_offset[1];
                let mut w2 = w2_row + step_x_offset[2];

                (start_x..end_x)
                    .skip(par_offset as usize * Edge::STEP_X)
                    .step_by(Edge::STEP_X as usize * par_count)
                    .for_each(|x| {
                        let pixel_mask = w0.cmple(*RENDER_MASK)
                            & w1.cmple(*RENDER_MASK)
                            & w2.cmple(*RENDER_MASK);

                        if pixel_mask.any() {
                            let weight_sum = w0 + w1 + w2;
                            let l1 = w1 / weight_sum;
                            let l2 = w2 / weight_sum;

                            let pz = Vec4::splat(triangle.points[0].z())
                                + (l1 * (triangle.points[1].z() - triangle.points[0].z()))
                                + (l2 * (triangle.points[2].z() - triangle.points[0].z()));

                            if let Some(depth_pass) =
                                (*ptr.0).try_set_depth_simd(x, y, pz, pixel_mask)
                            {
                                (*ptr.0).set_pixels_unchecked(x as u32, y, depth_pass, color);
                            }
                        }

                        w0 += step_x_count[0];
                        w1 += step_x_count[1];
                        w2 += step_x_count[2];
                    });

                w0_row += e12.one_step_y;
                w1_row += e20.one_step_y;
                w2_row += e01.one_step_y;
            });
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

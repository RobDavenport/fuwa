use super::{RasterBoundingBox, SlabPtr};
use crate::FSInput;
use crate::{FuwaPtr, Triangle};
use bytemuck::cast;
use glam::*;
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;
//use rayon::prelude::*;
use smallvec::SmallVec;
use wide::{f32x4, f32x8};

lazy_static! {
    static ref STAMP_OFFSET_X: f32x8 = f32x8::from([0., 1., 2., 3., 4., 5., 6., 7.]);
    static ref DEPTH_FAIL: f32x8 = f32x8::splat(f32::NAN);
    static ref STAMP_OFFSET_Y: f32x8 = f32x8::ZERO;
}

const INNER_STAMP_WIDTH: u32 = 8;
const INNER_STAMP_HEIGHT: u32 = 1;
const OUTER_BLOCK_WIDTH: u32 = 16;
const OUTER_BLOCK_HEIGHT: u32 = 16;

pub(crate) fn triangle<F: FSInput, W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<W>,
    triangle: &Triangle<F>,
    fs_index: usize,
    slab_ptr: SlabPtr<F>,
) {
    let points2d = triangle.get_points_as_vec2();
    let bb = unsafe { (*fuwa.0).calculate_raster_bb(&points2d) };

    rasterize_triangle(fuwa, triangle, fs_index, bb, slab_ptr)
}

fn rasterize_triangle<F: FSInput, W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<W>,
    triangle: &Triangle<F>,
    fs_index: usize,
    bb: RasterBoundingBox,
    slab_ptr: SlabPtr<F>,
) {
    //Triangle Setup
    let [v0, v1, v2] = triangle.get_points_as_vec3a();
    let origin = vec3a(bb.min_x(), bb.min_y(), 0.);

    let (mut w0_row, e12, a0) = prepare_edge(&v1, &v2, &origin);
    let (mut w1_row, e20, a1) = prepare_edge(&v2, &v0, &origin);
    let (mut w2_row, e01, a2) = prepare_edge(&v0, &v1, &origin);

    let one_over_area = (a0 + a1 + a2).recip();
    let z0_fast = v0.z();
    let z1_fast = (v1.z() - z0_fast) * one_over_area;
    let z2_fast = (v2.z() - z0_fast) * one_over_area;

    let f0_fast = triangle.vs_input[0];
    let f1_fast = (triangle.vs_input[1] - f0_fast) * one_over_area;
    let f2_fast = (triangle.vs_input[2] - f0_fast) * one_over_area;

    let [min_x, min_y, max_x, max_y] = bb.prepare();

    //Start rasterization
    for pixel_y in (min_y..max_y).step_by(INNER_STAMP_HEIGHT as usize) {
        let mut already_drew_row = false;
        //Set barycentric coordinates at start of row
        let mut w0 = w0_row;
        let mut w1 = w1_row;
        let mut w2 = w2_row;

        for pixel_x in (min_x..max_x).step_by(INNER_STAMP_WIDTH as usize) {
            let inside_triangle =
                w0.cmp_ge(f32x8::ZERO) & w1.cmp_ge(f32x8::ZERO) & w2.cmp_ge(f32x8::ZERO);

            if inside_triangle.any() {
                unsafe {
                    let depths = z0_fast + (w1 * z1_fast) + (w2 * z2_fast);
                    let pass_depths = inside_triangle.blend(depths, *DEPTH_FAIL);
                    already_drew_row = true;
                    if let Some(depth_mask) =
                        (*fuwa.0).try_set_depth_simd(pixel_x, pixel_y, &pass_depths)
                    {
                        let render_mask = depth_mask.move_mask();
                        let z_recip = cast::<_, [f32; 8]>(1. / depths);
                        let w1_vec = cast::<_, [f32; 8]>(w1);
                        let w2_vec = cast::<_, [f32; 8]>(w2);

                        for i in 0..8 {
                            if (1 << i) & render_mask != 0 {
                                let interped =
                                    (f0_fast + (f1_fast * w1_vec[i]) + (f2_fast * w2_vec[i]))
                                        * z_recip[i];
                                let fragment_key = slab_ptr.insert_fragment(fs_index, interped);
                                (*fuwa.0).set_fragment(pixel_x + i as u32, pixel_y, fragment_key);
                            }
                        }
                    }
                }
            } else {
                if already_drew_row {
                    break;
                }
            }

            //One step right
            w0 += e12.one_step_x;
            w1 += e20.one_step_x;
            w2 += e01.one_step_x;
        }

        //One row step
        w0_row += e12.one_step_y;
        w1_row += e20.one_step_y;
        w2_row += e01.one_step_y;
    }
}

struct EdgeStepper {
    one_step_x: f32x8,
    one_step_y: f32x8,
}

fn prepare_edge(v0: &Vec3A, v1: &Vec3A, origin: &Vec3A) -> (f32x8, EdgeStepper, f32) {
    // Edge setup
    let a = v1.y() - v0.y();
    let b = v0.x() - v1.x();
    let c = (v1.x() * v0.y()) - (v1.y() * v0.x());

    //Calculate initial values
    let x_start = f32x8::splat(origin.x()) + *STAMP_OFFSET_X;
    let y_start = f32x8::splat(origin.y()) + *STAMP_OFFSET_Y;

    //Actual edge value at origin
    let values = (f32x8::splat(a) * x_start) + (f32x8::splat(b) * y_start) + f32x8::splat(c);

    (
        values,
        EdgeStepper {
            one_step_x: f32x8::splat(a * INNER_STAMP_WIDTH as f32),
            one_step_y: f32x8::splat(b * INNER_STAMP_HEIGHT as f32),
        },
        a + b + c,
    )
}

//  Traverse outer blocks
//  Get barycentric coordinate for block corner
//    Match (check(v0), check(v1), check(v2)) {
//        (Outside, _, _) | (_, Outside, _) | (_, _, Outside) => We can early out,
//        (Inside, Inside, Inside) => Fast Rasterize Whole Block,
//        _ => Have to go pixel_by_pixel,
//    }

// enum BlockEdgeResult {
//     Outside,
//     Inside,
//     Partial,
// }

// impl BlockEdgeResult {
//     fn check(edge: &f32x4) -> Self {
//         match cast::<_, [i32; 4]>(edge.cmp_ge(f32x4::ZERO)) {
//             [-1, -1, -1, -1] => Self::Inside,
//             [0, 0, 0, 0] => Self::Outside,
//             _ => Self::Partial,
//         }
//     }
// }

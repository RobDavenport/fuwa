use super::{MapPtr, RasterBoundingBox};
use crate::{FSInput, FragmentShader};
use crate::{FuwaPtr, Triangle};
use bytemuck::cast;
use glam::*;
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;
use wide::{f32x4, f32x8};

lazy_static! {
    static ref STAMP_OFFSET_X: f32x8 = f32x8::from([0., 1., 2., 3., 4., 5., 6., 7.]);
    static ref DEPTH_MAX: f32x8 = f32x8::splat(f32::NAN);
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
    map_ptr: MapPtr<F>,
) {
    let points2d = triangle.get_points_as_vec2();
    let bb = unsafe { (*fuwa.0).calculate_raster_bb(&points2d) };

    rasterize_triangle_blocks(fuwa, triangle, fs_index, bb, map_ptr)
}

fn rasterize_triangle_blocks<F: FSInput, W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<W>,
    triangle: &Triangle<F>,
    fs_index: usize,
    bb: RasterBoundingBox,
    map_ptr: MapPtr<F>,
) {
    //optick::event!();
    let points = triangle.get_points_as_vec3a();

    //Deltas - Number indicates the edge
    let dx01 = points[0].x() - points[1].x();
    let dx12 = points[1].x() - points[2].x();
    let dx20 = points[2].x() - points[0].x();
    let dy01 = points[0].y() - points[1].y();
    let dy12 = points[1].y() - points[2].y();
    let dy20 = points[2].y() - points[0].y();

    //Constants - Number indicates the 'opposite edge'
    let c0 = dy12 * points[1].x() - dx12 * points[1].y();
    let c1 = dy20 * points[2].x() - dx20 * points[2].y();
    let c2 = dy01 * points[0].x() - dx01 * points[0].y();

    let [min_x, min_y, max_x, max_y] = bb.prepare();
    //Start traversing inner blocks
    //This can be done in parallel
    (min_y..max_y)
        .into_par_iter() //MAKE PAR
        .step_by(OUTER_BLOCK_HEIGHT as usize)
        .for_each(|block_y0| {
            //optick::register_thread("raster_tri_row");
            //Simple easy out per row
            let mut row_already_draw = false;

            for block_x0 in (min_x..max_x).step_by(OUTER_BLOCK_WIDTH as usize) {
                //Get block coordinates
                let block_x1 = block_x0 + OUTER_BLOCK_WIDTH;
                let block_y1 = block_y0 + OUTER_BLOCK_HEIGHT;

                //TODO: Optimize this with a single edge check?
                //Evaluate half-space for block edges
                let y_vec = f32x4::from([
                    block_y0 as f32,
                    block_y0 as f32,
                    block_y1 as f32,
                    block_y1 as f32,
                ]);
                let x_vec = f32x4::from([
                    block_x0 as f32,
                    block_x1 as f32,
                    block_x0 as f32,
                    block_x1 as f32,
                ]);
                let a_set = c0 + (dx12 * y_vec) - (dy12 * x_vec);
                let b_set = c1 + (dx20 * y_vec) - (dy20 * x_vec);
                let c_set = c2 + (dx01 * y_vec) - (dy01 * x_vec);

                use BlockEdgeResult::*;
                match (
                    BlockEdgeResult::check(&a_set),
                    BlockEdgeResult::check(&b_set),
                    BlockEdgeResult::check(&c_set),
                ) {
                    //Just skip any blocks completely outside
                    (Outside, _, _) | (_, Outside, _) | (_, _, Outside) => {
                        if row_already_draw {
                            return;
                        }
                    }

                    //TODO: SIMD-ify this block next
                    //We can draw this block in one go
                    (Inside, Inside, Inside) => {
                        row_already_draw = true;

                        let a00 = cast::<_, [f32; 4]>(a_set)[0];
                        let b00 = cast::<_, [f32; 4]>(b_set)[0];
                        let c00 = cast::<_, [f32; 4]>(c_set)[0];

                        let depths = get_interpolated_z_block(
                            triangle,
                            (a00, b00, c00),
                            (dx12, dx20, dx01),
                            (dy12, dy20, dy01),
                            OUTER_BLOCK_WIDTH,
                            OUTER_BLOCK_HEIGHT,
                        );

                        unsafe {
                            if let Some(depth_pass) = (*fuwa.0).try_set_depth_block(
                                (block_x0, block_y0),
                                (OUTER_BLOCK_WIDTH, OUTER_BLOCK_HEIGHT),
                                depths,
                            ) {
                                let interpolated_verts = get_interpolated_triangle_block(
                                    triangle,
                                    (a00, b00, c00),
                                    (dx12, dx20, dx01),
                                    (dy12, dy20, dy01),
                                    (OUTER_BLOCK_WIDTH, OUTER_BLOCK_HEIGHT),
                                    &depth_pass,
                                );
                                (*fuwa.0).set_fragments_block(
                                    (block_x0, block_y0),
                                    (OUTER_BLOCK_WIDTH, OUTER_BLOCK_HEIGHT),
                                    &depth_pass,
                                    interpolated_verts,
                                    fs_index,
                                    map_ptr,
                                )
                            }
                        }
                    }

                    _ => {
                        row_already_draw = true;
                        //We have a partially covered block, so we
                        //have to draw the block pixel-by-pixel
                        //These are constants for our new starting point at bx0, by0
                        //and were calculated previously
                        let mut cy0 = c0 + (dx12 * (block_y0 as f32 + *STAMP_OFFSET_Y))
                            - (dy12 * (block_x0 as f32 + *STAMP_OFFSET_X));
                        let mut cy1 = c1 + (dx20 * (block_y0 as f32 + *STAMP_OFFSET_Y))
                            - (dy20 * (block_x0 as f32 + *STAMP_OFFSET_X));
                        let mut cy2 = c2 + (dx01 * (block_y0 as f32 + *STAMP_OFFSET_Y))
                            - (dy01 * (block_x0 as f32 + *STAMP_OFFSET_X));

                        let one_step_x0 = f32x8::splat(dy12 * INNER_STAMP_WIDTH as f32);
                        let one_step_x1 = f32x8::splat(dy20 * INNER_STAMP_WIDTH as f32);
                        let one_step_x2 = f32x8::splat(dy01 * INNER_STAMP_WIDTH as f32);

                        let one_step_y0 = f32x8::splat(dx12 * INNER_STAMP_HEIGHT as f32);
                        let one_step_y1 = f32x8::splat(dx20 * INNER_STAMP_HEIGHT as f32);
                        let one_step_y2 = f32x8::splat(dx01 * INNER_STAMP_HEIGHT as f32);

                        (block_y0..block_y1)
                            .step_by(INNER_STAMP_HEIGHT as usize)
                            .for_each(|pixel_y| {
                                //Reset values for horizontal traversal
                                let mut cx0 = cy0;
                                let mut cx1 = cy1;
                                let mut cx2 = cy2;

                                (block_x0..block_x1)
                                    .step_by(INNER_STAMP_WIDTH as usize)
                                    .for_each(|pixel_x| {
                                        let tri_mask = cx0.cmp_ge(f32x8::ZERO)
                                            & cx1.cmp_ge(f32x8::ZERO)
                                            & cx2.cmp_ge(f32x8::ZERO);
                                        if tri_mask.any() {
                                            let pixel_zs = tri_mask.blend(
                                                get_interpolated_z_simd(
                                                    &triangle, &cx0, &cx1, &cx2,
                                                ),
                                                *DEPTH_MAX,
                                            );
                                            unsafe {
                                                if let Some(depth_pass) = (*fuwa.0)
                                                    .try_set_depth_simd(pixel_x, pixel_y, &pixel_zs)
                                                {
                                                    let interpolants = interpolate_triangle_simd(
                                                        &triangle, &cx0, &cx1, &cx2, pixel_zs,
                                                    );
                                                    (*fuwa.0).set_fragments_simd(
                                                        pixel_x,
                                                        pixel_y,
                                                        interpolants,
                                                        depth_pass,
                                                        fs_index,
                                                        map_ptr,
                                                    );
                                                }
                                            }
                                        }
                                        cx0 -= one_step_x0;
                                        cx1 -= one_step_x1;
                                        cx2 -= one_step_x2;
                                    });
                                cy0 += one_step_y0;
                                cy1 += one_step_y1;
                                cy2 += one_step_y2;
                            })
                    }
                }
            }
        });
}
enum BlockEdgeResult {
    Outside,
    Inside,
    Partial,
}

impl BlockEdgeResult {
    fn check(edge: &f32x4) -> Self {
        match cast::<_, [i32; 4]>(edge.cmp_ge(f32x4::ZERO)) {
            [-1, -1, -1, -1] => Self::Inside,
            [0, 0, 0, 0] => Self::Outside,
            _ => Self::Partial,
        }
    }
}

fn get_interp_values(w0: f32, w1: f32, w2: f32) -> (f32, f32) {
    let weight_sum = w0 + w1 + w2;
    let l1 = w1 / weight_sum;
    let l2 = w2 / weight_sum;
    (l1, l2)
}

fn get_interp_values_simd(w0: &f32x8, w1: &f32x8, w2: &f32x8) -> (f32x8, f32x8) {
    let weight_sum = *w0 + *w1 + *w2;
    let l1 = *w1 / weight_sum;
    let l2 = *w2 / weight_sum;
    (l1, l2)
}

//TODO: FIX THIS LATER
// fn get_interpolated_z(triangle: &Triangle, w0: f32, w1: f32, w2: f32) -> f32 {
//     //optick::event!();

//     let (l1, l2) = get_interp_values(w0, w1, w2);
//     let [z0, zs10, zs20] = triangle.get_z_diffs();
//     z0 + (l1 * zs10) + (l2 * zs20)
// }

fn get_interpolated_z_simd<F: FSInput>(
    triangle: &Triangle<F>,
    w0: &f32x8,
    w1: &f32x8,
    w2: &f32x8,
) -> f32x8 {
    //optick::event!();

    let (l1, l2) = get_interp_values_simd(w0, w1, w2);
    let [z0, zs10, zs20] = triangle.get_z_diffs();
    *z0 + (l1 * *zs10) + (l2 * *zs20)
}

//TODO: FIX THIS LATER
// fn interpolate_triangle(triangle: &Triangle, w0: f32, w1: f32, w2: f32, pixel_z: f32) -> Vec<f32> {
//     //optick::event!();

//     let (l1, l2) = get_interp_values(w0, w1, w2);
//     let [p0, sub10, sub20] = triangle.get_interpolate_diffs();
//     let len = p0.len();

//     let mut out = Vec::with_capacity(len);
//     for i in 0..len {
//         out.push((p0[i] + (l1 * sub10[i]) + (l2 * sub20[i])) * pixel_z);
//     }
//     out
// }

fn interpolate_triangle_simd<F: FSInput>(
    triangle: &Triangle<F>,
    w0: &f32x8,
    w1: &f32x8,
    w2: &f32x8,
    pixel_zs: f32x8,
) -> [F; 8] {
    //optick::event!();
    let (l1, l2) = get_interp_values_simd(w0, w1, w2);
    let [p0, sub10, sub20] = triangle.get_interpolate_diffs();

    let pixel_zs = cast::<_, [f32; 8]>(1. / pixel_zs);
    let l1_vec = cast::<_, [f32; 8]>(l1);
    let l2_vec = cast::<_, [f32; 8]>(l2);

    [
        (*p0 + (*sub10 * l1_vec[0]) + (*sub20 * l2_vec[0])) * pixel_zs[0],
        (*p0 + (*sub10 * l1_vec[1]) + (*sub20 * l2_vec[1])) * pixel_zs[1],
        (*p0 + (*sub10 * l1_vec[2]) + (*sub20 * l2_vec[2])) * pixel_zs[2],
        (*p0 + (*sub10 * l1_vec[3]) + (*sub20 * l2_vec[3])) * pixel_zs[3],
        (*p0 + (*sub10 * l1_vec[4]) + (*sub20 * l2_vec[4])) * pixel_zs[4],
        (*p0 + (*sub10 * l1_vec[5]) + (*sub20 * l2_vec[5])) * pixel_zs[5],
        (*p0 + (*sub10 * l1_vec[6]) + (*sub20 * l2_vec[6])) * pixel_zs[6],
        (*p0 + (*sub10 * l1_vec[7]) + (*sub20 * l2_vec[7])) * pixel_zs[7],
    ]
}

fn get_interpolated_z_block<F: FSInput>(
    triangle: &Triangle<F>,
    w00: (f32, f32, f32),
    dx: (f32, f32, f32),
    dy: (f32, f32, f32),
    width: u32,
    height: u32,
) -> Vec<f32> {
    //optick::event!();
    let [z0, zs10, zs20] = triangle.get_z_diffs();

    let mut left_interpolator = Vec3A::from(w00);
    let step_x = Vec3A::from(dy);
    let step_y = Vec3A::from(dx);

    let mut out = Vec::with_capacity((width * height) as usize);
    for _ in 0..height {
        let mut x_interpolator = left_interpolator;
        for _ in 0..width {
            let (l1, l2) =
                get_interp_values(x_interpolator[0], x_interpolator[1], x_interpolator[2]);
            out.push(z0 + (l1 * zs10) + (l2 * zs20));
            x_interpolator -= step_x;
        }
        left_interpolator += step_y;
    }
    out
}

//TODO: Perf this
fn get_interpolated_triangle_block<F: FSInput>(
    triangle: &Triangle<F>,
    w00: (f32, f32, f32),
    dx: (f32, f32, f32),
    dy: (f32, f32, f32),
    (width, height): (u32, u32),
    depth_pass: &[Option<f32>],
) -> Vec<F> {
    let mut left_interpolator = Vec3A::from(w00);
    let step_x = Vec3A::from(dy);
    let step_y = Vec3A::from(dx);
    let mut counter: usize = 0;
    let [p0s, sub10, sub20] = triangle.get_interpolate_diffs();

    let mut out = Vec::with_capacity((width * height) as usize);
    for _ in 0..height {
        let mut x_interpolator = left_interpolator;
        for _ in 0..width {
            if let Some(pixel_depth) = depth_pass[counter] {
                let pixel_depth = pixel_depth.recip();
                let (l1, l2) =
                    get_interp_values(x_interpolator[0], x_interpolator[1], x_interpolator[2]);
                out.push((*p0s + (*sub10 * l1) + (*sub20 * l2)) * pixel_depth);
            }
            x_interpolator -= step_x;
            counter += 1;
        }
        left_interpolator += step_y;
    }
    out
}

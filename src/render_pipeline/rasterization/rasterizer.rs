use super::{Fragment, RasterBoundingBox};
use crate::{FragmentShaderFunction, FuwaPtr, Triangle};
use glam::*;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;
use wide::{f32x4, f32x8};
use bytemuck::cast;

const BLOCK_WIDTH: u32 = 8;
const BLOCK_HEIGHT: u32 = 8;

pub(crate) fn triangle<'fs, W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<'fs, W>,
    triangle: &Triangle,
    fs: &'fs FragmentShaderFunction,
) {
    let points2d = triangle.get_points_as_vec2();
    let bb = unsafe { (*fuwa.0).calculate_raster_bb(&points2d) };

    rasterize_triangle_blocks(fuwa, triangle, fs, bb)
}

fn rasterize_triangle_blocks<'fs, W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<'fs, W>,
    triangle: &Triangle,
    fs: &'fs FragmentShaderFunction,
    bb: RasterBoundingBox,
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
        .into_par_iter()
        .step_by(BLOCK_HEIGHT as usize)
        .for_each(|block_y0| {
            //optick::register_thread("raster_tri_row");
            //Simple easy out per row
            let mut row_already_draw = false;

            (min_x..max_x)
                .step_by(BLOCK_WIDTH as usize)
                .for_each(|block_x0| {
                    //Get block coordinates
                    let block_x1 = block_x0 + BLOCK_WIDTH;
                    let block_y1 = block_y0 + BLOCK_HEIGHT;

                    //TODO: Optimize this with a single edge check?
                    //Evaluate half-space for block edges
                    let y_vec = f32x4::from([block_y0 as f32, block_y0 as f32, block_y1 as f32, block_y1 as f32]);
                    let x_vec = f32x4::from([block_x0 as f32, block_x1 as f32, block_x0 as f32, block_x1 as f32]);
                    let a_set = c0 + (dx12 * y_vec) - (dy12 * x_vec);
                    let b_set = c1 + (dx20 * y_vec) - (dy20 * x_vec);
                    let c_set = c2 + (dx01 * y_vec) - (dy01 * x_vec);

                    use BlockEdgeResult::*;
                    match (BlockEdgeResult::check(&a_set), BlockEdgeResult::check(&b_set), BlockEdgeResult::check(&c_set)) {
                        //Just skip any blocks completely outside
                        (Outside, Outside, Outside) => {
                            if row_already_draw {
                                return;
                            }
                        }

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
                                BLOCK_WIDTH,
                                BLOCK_HEIGHT,
                            );

                            unsafe {
                                if let Some(depth_pass) = (*fuwa.0).try_set_depth_block(
                                    (block_x0, block_y0),
                                    (BLOCK_WIDTH, BLOCK_HEIGHT),
                                    depths,
                                ) {
                                    let interpolated_verts = get_interpolated_triangle_block(
                                        triangle,
                                        (a00, b00, c00),
                                        (dx12, dx20, dx01),
                                        (dy12, dy20, dy01),
                                        (BLOCK_WIDTH, BLOCK_HEIGHT),
                                        &depth_pass,
                                    );
                                    (*fuwa.0).set_fragments_block(
                                        (block_x0, block_y0),
                                        (BLOCK_WIDTH, BLOCK_HEIGHT),
                                        &depth_pass,
                                        interpolated_verts,
                                        fs,
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
                            let a00 = cast::<_, [f32; 4]>(a_set)[0];
                            let b00 = cast::<_, [f32; 4]>(b_set)[0];
                            let c00 = cast::<_, [f32; 4]>(c_set)[0];

                            let mut cy0 = a00;
                            let mut cy1 = b00;
                            let mut cy2 = c00;

                            (block_y0..block_y1).for_each(|pixel_y| {
                                //Reset values for horizontal traversal
                                let mut cx0 = cy0;
                                let mut cx1 = cy1;
                                let mut cx2 = cy2;

                                (block_x0..block_x1).for_each(|pixel_x| {
                                    //Check if pixel is inside the triangle
                                    //We can just check the sign bit because they are floats.
                                    // >= 0 means all values are positive
                                    if ((cx0.to_bits() | cx1.to_bits() | cx2.to_bits()) as i32) >= 0
                                    {
                                        let pz = get_interpolated_z(&triangle, cx0, cx1, cx2);
                                        unsafe {
                                            if (*fuwa.0).try_set_depth(pixel_x, pixel_y, pz) {
                                                let interpolants = interpolate_triangle(
                                                    &triangle,
                                                    cx0,
                                                    cx1,
                                                    cx2,
                                                    pz.recip(),
                                                );
                                                (*fuwa.0).set_fragment(
                                                    pixel_x,
                                                    pixel_y,
                                                    Fragment {
                                                        interpolants,
                                                        shader: fs,
                                                    },
                                                );
                                            }
                                        }
                                    }

                                    cx0 -= dy12;
                                    cx1 -= dy20;
                                    cx2 -= dy01;
                                });

                                cy0 += dx12;
                                cy1 += dx20;
                                cy2 += dx01;
                            })
                        }
                    }
                })
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

fn get_interpolated_z(triangle: &Triangle, w0: f32, w1: f32, w2: f32) -> f32 {
    //optick::event!();

    let (l1, l2) = get_interp_values(w0, w1, w2);
    let [z0, zs10, zs20] = triangle.get_z_diffs();
    z0 + (l1 * zs10) + (l2 * zs20)
}

//TODO: Can I use SIMD here?
fn interpolate_triangle(triangle: &Triangle, w0: f32, w1: f32, w2: f32, pixel_z: f32) -> Vec<f32> {
    //optick::event!();

    let (l1, l2) = get_interp_values(w0, w1, w2);
    let [p0, sub10, sub20] = triangle.get_interpolate_diffs();
    let len = p0.len();

    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push((p0[i] + (l1 * sub10[i]) + (l2 * sub20[i])) * pixel_z);
    }
    out
}

fn get_interpolated_z_block(
    triangle: &Triangle,
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
fn get_interpolated_triangle_block(
    triangle: &Triangle,
    w00: (f32, f32, f32),
    dx: (f32, f32, f32),
    dy: (f32, f32, f32),
    (width, height): (u32, u32),
    depth_pass: &[Option<f32>],
) -> Vec<Vec<f32>> {
    //optick::event!();

    let mut left_interpolator = Vec3A::from(w00);
    let step_x = Vec3A::from(dy);
    let step_y = Vec3A::from(dx);
    let mut counter: usize = 0;
    let [p0s, sub10, sub20] = triangle.get_interpolate_diffs();
    let len = p0s.len();

    let mut out = Vec::with_capacity((width * height) as usize);
    for _ in 0..height {
        let mut x_interpolator = left_interpolator;
        for _ in 0..width {
            if let Some(pixel_depth) = depth_pass[counter] {
                let mut inner = Vec::with_capacity(len);
                let (l1, l2) =
                    get_interp_values(x_interpolator[0], x_interpolator[1], x_interpolator[2]);
                for i in 0..len {
                    inner.push((p0s[i] + (l1 * sub10[i]) + (l2 * sub20[i])) * pixel_depth);
                }
                out.push(inner);
            }
            x_interpolator -= step_x;
            counter += 1;
        }
        left_interpolator += step_y;
    }
    out
}

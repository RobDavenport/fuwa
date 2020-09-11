use super::RasterBoundingBox;
use crate::{FragmentShaderFunction, FuwaPtr, Triangle};
use glam::*;
use itertools::izip;
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;

const BLOCK_WIDTH: u32 = 8;
const BLOCK_HEIGHT: u32 = 8;

pub(crate) fn triangle<W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<W>,
    triangle: &Triangle,
    fs: &FragmentShaderFunction,
) {
    let points2d = triangle.get_points_as_vec2();
    let bb = unsafe { (*fuwa.0).calculate_raster_bb(&points2d) };

    rasterize_triangle_blocks(fuwa, triangle, fs, bb)
}

fn rasterize_triangle_blocks<W: HasRawWindowHandle + Send + Sync>(
    fuwa: FuwaPtr<W>,
    triangle: &Triangle,
    fs: &FragmentShaderFunction,
    bb: RasterBoundingBox,
) {
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
                    //Rembeber, the C edge function uses opposite point indexes
                    //Edge A - 0->1
                    let a00 = c0 + (dx12 * block_y0 as f32) - (dy12 * block_x0 as f32);
                    let a10 = c0 + (dx12 * block_y0 as f32) - (dy12 * block_x1 as f32);
                    let a01 = c0 + (dx12 * block_y1 as f32) - (dy12 * block_x0 as f32);
                    let a11 = c0 + (dx12 * block_y1 as f32) - (dy12 * block_x1 as f32);
                    let a_in = BlockEdgeResult::check(a00, a01, a10, a11);

                    //Edge B - 1->2
                    let b00 = c1 + (dx20 * block_y0 as f32) - (dy20 * block_x0 as f32);
                    let b10 = c1 + (dx20 * block_y0 as f32) - (dy20 * block_x1 as f32);
                    let b01 = c1 + (dx20 * block_y1 as f32) - (dy20 * block_x0 as f32);
                    let b11 = c1 + (dx20 * block_y1 as f32) - (dy20 * block_x1 as f32);
                    let b_in = BlockEdgeResult::check(b00, b01, b10, b11);

                    //Edge C - 2->0
                    let c00 = c2 + (dx01 * block_y0 as f32) - (dy01 * block_x0 as f32);
                    let c10 = c2 + (dx01 * block_y0 as f32) - (dy01 * block_x1 as f32);
                    let c01 = c2 + (dx01 * block_y1 as f32) - (dy01 * block_x0 as f32);
                    let c11 = c2 + (dx01 * block_y1 as f32) - (dy01 * block_x1 as f32);
                    let c_in = BlockEdgeResult::check(c00, c01, c10, c11);

                    use BlockEdgeResult::*;
                    match (a_in, b_in, c_in) {
                        //Just skip any blocks completely outside
                        (Outside, Outside, Outside) => {
                            if row_already_draw {
                                return;
                            }
                        }

                        //We can draw this block in one go
                        (Inside, Inside, Inside) => {
                            row_already_draw = true;

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
                                    (*fuwa.0).set_pixels_block(
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
                                                let interpolated_data = &interpolate_triangle(
                                                    &triangle,
                                                    cx0,
                                                    cx1,
                                                    cx2,
                                                    pz.recip(),
                                                );
                                                (*fuwa.0).set_pixel_unchecked(
                                                    pixel_x,
                                                    pixel_y,
                                                    &(fs)(interpolated_data, &(*fuwa.0).uniforms),
                                                )
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
    fn check(p00: f32, p01: f32, p10: f32, p11: f32) -> Self {
        match (
            p00.is_sign_positive(),
            p01.is_sign_positive(),
            p10.is_sign_positive(),
            p11.is_sign_positive(),
        ) {
            (true, true, true, true) => Self::Inside,
            (false, false, false, false) => Self::Outside,
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
    let (l1, l2) = get_interp_values(w0, w1, w2);
    let z_index = triangle.position_index + 2;
    let z0 = triangle.points[0].raw_data[z_index];
    let z1 = triangle.points[1].raw_data[z_index];
    let z2 = triangle.points[2].raw_data[z_index];
    z0 + (l1 * (z1 - z0)) + (l2 * (z2 - z0))
}

//TODO: Can I use SIMD here?
fn interpolate_triangle(triangle: &Triangle, w0: f32, w1: f32, w2: f32, pixel_z: f32) -> Vec<f32> {
    let (l1, l2) = get_interp_values(w0, w1, w2);
    let (p0i, p1i, p2i, len) = triangle.get_vertex_iterators();

    let mut out = Vec::with_capacity(len);
    for (p0, p1, p2) in izip!(p0i, p1i, p2i) {
        out.push((p0 + (l1 * (p1 - p0)) + (l2 * (p2 - p0))) * pixel_z);
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
    let z_index = triangle.position_index + 2;
    let z0 = triangle.points[0].raw_data[z_index];
    let z1 = triangle.points[1].raw_data[z_index];
    let z2 = triangle.points[2].raw_data[z_index];
    let zs10 = z1 - z0;
    let zs20 = z2 - z0;

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

fn get_interpolated_triangle_block(
    triangle: &Triangle,
    w00: (f32, f32, f32),
    dx: (f32, f32, f32),
    dy: (f32, f32, f32),
    (width, height): (u32, u32),
    depth_pass: &[Option<f32>],
) -> Vec<Vec<f32>> {
    let mut left_interpolator = Vec3A::from(w00);
    let step_x = Vec3A::from(dy);
    let step_y = Vec3A::from(dx);
    let (p0i, p1i, p2i, len) = triangle.get_vertex_iterators();
    let mut counter: usize = 0;

    let mut p0s = Vec::with_capacity(len);
    let mut sub10 = Vec::with_capacity(len);
    let mut sub20 = Vec::with_capacity(len);

    for (p0, p1, p2) in izip!(p0i, p1i, p2i) {
        p0s.push(p0);
        sub10.push(p1 - p0);
        sub20.push(p2 - p0);
    }

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

use super::handle::Handle;
use super::rasterizer::Edge;
use super::HandleGenerator;
use super::Texture;
use glam::*;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;
use std::marker::{Send, Sync};

pub struct Fuwa<W: HasRawWindowHandle> {
    pub width: u32,
    pub height: u32,
    pub pixel_count: u32,
    pub x_factor: f32,
    pub y_factor: f32,
    pixels: Pixels<W>,
    depth_buffer: Vec<f32>,
    pub raster_par_count: usize,
    pub(crate) textures: HashMap<Handle<Texture>, Texture>,
    pub(crate) texture_generator: HandleGenerator<Texture>,
}

lazy_static! {
    static ref RENDER_MASK: Vec4 = Vec4::new(0., 0., 0., 0.);
    static ref THREAD_COLOR: [&'static [u8; 4]; 8] = [
        &super::Colors::RED,
        &super::Colors::GREEN,
        &super::Colors::BLUE,
        &super::Colors::CYAN,
        &super::Colors::MAGENTA,
        &super::Colors::YELLOW,
        &super::Colors::PINK,
        &super::Colors::OFFWHITE,
    ];
}

#[derive(Copy, Clone)]
struct FuwaPtr<W: HasRawWindowHandle>(*mut Fuwa<W>);

unsafe impl<W: HasRawWindowHandle> Send for FuwaPtr<W> {}
unsafe impl<W: HasRawWindowHandle> Sync for FuwaPtr<W> {}

impl<W: HasRawWindowHandle + Send + Sync> Fuwa<W> {
    fn get_self_ptr(&mut self) -> FuwaPtr<W> {
        FuwaPtr(self as *mut Self)
    }

    pub fn new(
        width: u32,
        height: u32,
        raster_par_count: usize,
        vsync: bool,
        high_performance: Option<bool>,
        window: &W,
    ) -> Self {
        Self {
            width,
            height,
            pixel_count: width * height,
            depth_buffer: vec![f32::INFINITY; (width * height) as usize],
            raster_par_count,
            x_factor: width as f32 * 0.5,
            y_factor: height as f32 * 0.5,
            textures: HashMap::new(),
            texture_generator: HandleGenerator::new(),
            pixels: PixelsBuilder::new(width, height, SurfaceTexture::new(width, height, &*window))
                .enable_vsync(vsync)
                .request_adapter_options(RequestAdapterOptions {
                    power_preference: match high_performance {
                        None => PowerPreference::Default,
                        Some(true) => PowerPreference::HighPerformance,
                        Some(false) => PowerPreference::LowPower,
                    },
                    compatible_surface: None,
                })
                .build()
                .unwrap(),
        }
    }

    pub fn clear(&mut self, color: &[u8; 4]) {
        let color_bytes: [u8; 64] = [
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3], color[0], color[1], color[2], color[3],
        ];
        self.pixels
            .get_frame()
            .par_chunks_exact_mut(4 * 16)
            .for_each(|pixel_chunk| {
                pixel_chunk.copy_from_slice(&color_bytes);
            })
    }

    pub fn try_set_depth(&mut self, x: u32, y: u32, depth: f32) -> bool {
        unsafe {
            let prev = self
                .depth_buffer
                .get_unchecked_mut((x + y * self.width) as usize);
            if depth < *prev {
                *prev = depth;
                true
            } else {
                false
            }
        }
    }

    pub fn try_set_depth_simd(
        &mut self,
        x: u32,
        y: u32,
        depths: Vec4,
        mask: Vec4Mask,
    ) -> Option<Vec4Mask> {
        unsafe {
            let index = (x + y * self.width) as usize;
            let values = self
                .depth_buffer
                .get_unchecked_mut(index..index + Edge::STEP_X);

            let prev_depths = vec4(values[0], values[1], values[2], values[3]);
            let result = depths.cmplt(prev_depths) & mask;

            if result.any() {
                let update = result.select(depths, prev_depths);
                values.copy_from_slice(&[update[0], update[1], update[2], update[3]]);
                Some(result)
            } else {
                None
            }
        }
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.par_iter_mut().for_each(|x| {
            *x = f32::INFINITY;
        });
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.pixels.render()
    }

    pub fn render_depth_buffer(&mut self) -> Result<(), Error> {
        let pixel_iter = self.pixels.get_frame().par_chunks_exact_mut(4);
        let mut depth_max = f32::NEG_INFINITY;
        let mut depth_min = f32::INFINITY;

        self.depth_buffer.iter().for_each(|x| {
            if x.is_finite() {
                if *x > depth_max {
                    depth_max = *x
                } else if *x < depth_min {
                    depth_min = *x
                }
            }
        });

        let range = depth_max - depth_min;
        let depth_iter = self.depth_buffer.par_iter_mut();

        if range != 0. {
            depth_iter.for_each(|x| {
                *x = u8::MAX as f32 - ((*x - depth_min) / range * u8::MAX as f32);
            });
        } else {
            depth_iter.for_each(|x| {
                if x.is_finite() {
                    *x = u8::MAX as f32;
                }
            });
        }

        let depth_iter = self.depth_buffer.par_iter_mut();
        pixel_iter.zip(depth_iter).for_each(|(pixel, depth)| {
            let color = *depth as u8;
            pixel.copy_from_slice(&[color, color, color, 0xFF]);
        });

        self.pixels.render()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize(width, height);
        self.pixel_count = width * height;
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        let index = self.pos_to_index(x, y);

        assert!(
            (index as u32) < self.pixel_count * 4,
            "Pixel drawn out of bounds!"
        );

        self.set_pixel_by_index(index, color)
    }

    fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        self.set_pixel_by_index(self.pos_to_index(x, y), color)
    }

    fn set_pixels_unchecked(&mut self, x: u32, y: u32, mask: Vec4Mask, color: &[u8; 4]) {
        let index = self.pos_to_index(x, y);

        unsafe {
            let color_float = f32::from_ne_bytes(*color);

            let current_pixels_ptr = self
                .pixels
                .get_frame()
                .get_unchecked_mut(index..index + 4 * Edge::STEP_X)
                .as_mut_ptr();

            let insert = mask.select(
                vec4(color_float, color_float, color_float, color_float),
                vec4_from_pixel_ptr(current_pixels_ptr as *const f32),
            );

            current_pixels_ptr.copy_from(
                insert.as_ref().as_ptr() as *const u8,
                (16 - (x.saturating_sub(self.width - Edge::STEP_X as u32) << 2)) as usize,
            );
        };
    }

    fn pos_to_index(&self, x: u32, y: u32) -> usize {
        4 * (x + (y * self.width)) as usize
    }

    fn set_pixel_by_index(&mut self, index: usize, color: &[u8; 4]) {
        unsafe {
            self.pixels
                .get_frame()
                .get_unchecked_mut(index..index + 4)
                .copy_from_slice(color);
        }
    }

    pub fn draw_triangle(&mut self, points: &[Vec3A; 3], color: &[u8; 4]) {
        let mut min_x = points[0].x();
        let mut min_y = points[0].y();

        let mut max_x = min_x;
        let mut max_y = min_y;

        for point in points.iter().skip(1) {
            let test_x = point.x();
            let test_y = point.y();

            if test_x < min_x {
                min_x = test_x
            } else if test_x > max_x {
                max_x = test_x;
            }

            if test_y < min_y {
                min_y = test_y
            } else if test_y > max_y {
                max_y = test_y
            }
        }

        //clamp to view
        if min_x < 0. {
            min_x = 0.;
        }

        if min_y < 0. {
            min_y = 0.;
        }

        if max_x > self.width as f32 - 1. {
            max_x = self.width as f32 - 1.;
        }

        if max_y > self.height as f32 - 1. {
            max_y = self.height as f32 - 1.;
        }

        if is_degenerate(points, Vec2::new(points[0].x(), points[0].y())) {
            return;
        }

        unsafe {
            let self_ptr = self.get_self_ptr();
            (min_y.floor() as u32..=max_y.ceil() as u32)
                .into_par_iter()
                .for_each(|y| {
                    (min_x.floor() as u32..=max_x.ceil() as u32)
                        .into_par_iter()
                        .for_each(|x| {
                            let barycentric = barycentric(points, Vec2::new(x as f32, y as f32));
                            if !(barycentric.x() < 0.
                                || barycentric.y() < 0.
                                || barycentric.z() < 0.)
                            {
                                (*self_ptr.0).set_pixel_unchecked(x, y, color);
                            }
                        });
                });
        }
    }

    pub fn draw_triangle_fast(&mut self, points: &[Vec3A; 3], color: &[u8; 4]) {
        let mut min_x = points[0].x();
        let mut min_y = points[0].y();

        let mut max_x = min_x;
        let mut max_y = min_y;

        for point in points.iter().skip(1) {
            let test_x = point.x();
            let test_y = point.y();

            if test_x < min_x {
                min_x = test_x
            } else if test_x > max_x {
                max_x = test_x;
            }

            if test_y < min_y {
                min_y = test_y
            } else if test_y > max_y {
                max_y = test_y
            }
        }

        //clamp to view
        if min_x < 0. {
            min_x = 0.;
        }

        if min_y < 0. {
            min_y = 0.;
        }

        if max_x >= self.width as f32 {
            max_x = self.width as f32 - 1.;
        }

        if max_y >= self.height as f32 {
            max_y = self.height as f32 - 1.;
        }

        min_x = min_x.floor();
        min_y = min_y.floor();
        max_x = max_x.ceil();
        max_y = max_y.ceil();

        let a01 = points[0].y() - points[1].y();
        let a12 = points[1].y() - points[2].y();
        let a20 = points[2].y() - points[0].y();

        let b01 = points[1].x() - points[0].x();
        let b12 = points[2].x() - points[1].x();
        let b20 = points[0].x() - points[2].x();

        let p = Vec3A::new(min_x, min_y, 0.0);

        let mut w0_row = orient_2d(&points[1], &points[2], &p);
        let mut w1_row = orient_2d(&points[2], &points[0], &p);
        let mut w2_row = orient_2d(&points[0], &points[1], &p);

        let self_ptr = self.get_self_ptr();
        unsafe {
            (min_y as u32..max_y as u32).for_each(|y| {
                let mut w0 = w0_row;
                let mut w1 = w1_row;
                let mut w2 = w2_row;

                (min_x as u32..max_x as u32).for_each(|x| {
                    if w0.is_sign_negative() && w1.is_sign_negative() && w2.is_sign_negative() {
                        let weight_sum = w0 + w1 + w2;
                        let l1 = w1 / weight_sum;
                        let l2 = w2 / weight_sum;

                        let pz = points[0].z()
                            + (l1 * (points[1].z() - points[0].z()))
                            + (l2 * (points[2].z() - points[0].z()));

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

    pub fn draw_triangle_parallel(&mut self, points: &[Vec3A; 3], color: &[u8; 4]) {
        let mut min_x = points[0].x();
        let mut min_y = points[0].y();

        let mut max_x = min_x;
        let mut max_y = min_y;

        for point in points.iter().skip(1) {
            let test_x = point.x();
            let test_y = point.y();

            if test_x < min_x {
                min_x = test_x
            } else if test_x > max_x {
                max_x = test_x;
            }

            if test_y < min_y {
                min_y = test_y
            } else if test_y > max_y {
                max_y = test_y
            }
        }

        //clamp to view
        if min_x < 0. {
            min_x = 0.;
        }

        if min_y < 0. {
            min_y = 0.;
        }

        if max_x >= self.width as f32 {
            max_x = self.width as f32 - 1.;
        }

        if max_y >= self.height as f32 {
            max_y = self.height as f32 - 1.;
        }

        min_x = min_x.floor();
        min_y = min_y.floor();
        max_x = max_x.ceil();
        max_y = max_y.ceil();

        let self_ptr = self.get_self_ptr();

        let p = Vec3A::new(min_x as f32, min_y as f32, 0.0);
        let e12 = Edge::init(&points[1], &points[2], &p);
        let e20 = Edge::init(&points[2], &points[0], &p);
        let e01 = Edge::init(&points[0], &points[1], &p);

        unsafe {
            // Self::rasterize_triangle(
            //     &self_ptr,
            //     (e12, e20, e01),
            //     (start_x as u32, start_y as u32),
            //     (end_x as u32, end_y as u32),
            //     (2, 4),
            //     color,
            // );
            (0..self.raster_par_count)
                .into_par_iter()
                .for_each(|thread_offset| {
                    Self::rasterize_triangle(
                        &self_ptr,
                        points,
                        (e12, e20, e01),
                        (min_x as u32, min_y as u32),
                        (max_x as u32, max_y as u32),
                        (thread_offset, self.raster_par_count),
                        color,
                    );
                });
        }
    }

    unsafe fn rasterize_triangle(
        ptr: &FuwaPtr<W>,
        points: &[Vec3A; 3],
        ((e12, mut w0_row), (e20, mut w1_row), (e01, mut w2_row)): (
            (Edge, Vec4),
            (Edge, Vec4),
            (Edge, Vec4),
        ),
        (start_x, start_y): (u32, u32),
        (end_x, end_y): (u32, u32),
        (par_offset, par_count): (usize, usize),
        color: &[u8; 4],
    ) {
        (start_y as u32..end_y as u32)
            .step_by(Edge::STEP_Y as usize)
            .for_each(|y| {
                let mut w0 = w0_row;
                let mut w1 = w1_row;
                let mut w2 = w2_row;

                (0..par_offset).for_each(|_| {
                    w0 += e12.one_step_x;
                    w1 += e20.one_step_x;
                    w2 += e01.one_step_x;
                });

                (start_x as u32..end_x as u32)
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

                            let pz =
                                vec4(points[0].z(), points[0].z(), points[0].z(), points[0].z())
                                    + (l1 * (points[1].z() - points[0].z()))
                                    + (l2 * (points[2].z() - points[0].z()));

                            if let Some(depth_pass) =
                                (*ptr.0).try_set_depth_simd(x, y, pz, pixel_mask)
                            {
                                (*ptr.0).set_pixels_unchecked(x as u32, y, depth_pass, color);
                            }
                        }

                        (0..par_count).for_each(|_| {
                            w0 += e12.one_step_x;
                            w1 += e20.one_step_x;
                            w2 += e01.one_step_x;
                        });
                    });

                w0_row += e12.one_step_y;
                w1_row += e20.one_step_y;
                w2_row += e01.one_step_y;
            });
    }

    pub fn draw_line(&mut self, mut start: Vec3A, mut end: Vec3A, color: &[u8; 4]) {
        use std::ptr::swap;

        assert!(
            self.check_3d_pixel_within_bounds(&start) && self.check_3d_pixel_within_bounds(&end),
            "Line drawn out of bounds."
        );

        let start_ptr = &mut start as *mut Vec3A;
        let end_ptr = &mut end as *mut Vec3A;
        let steep = if f32::abs(start.x() - end.x()) < f32::abs(start.y() - end.y()) {
            unsafe {
                swap((*start_ptr).x_mut(), (*start_ptr).y_mut());
                swap((*end_ptr).x_mut(), (*end_ptr).y_mut());
            }
            true
        } else {
            false
        };

        if start.x() > end.x() {
            unsafe {
                swap(start_ptr, end_ptr);
            }
        }

        unsafe {
            let self_ptr = self.get_self_ptr();

            (start.x() as u32..=end.x() as u32)
                .into_par_iter()
                .for_each(|x| {
                    let slope = (x as f32 - start.x()) / (end.x() - start.x());
                    let y = (start.y() * (1. - slope) + end.y() * slope) as u32;
                    if steep {
                        (*self_ptr.0).set_pixel_unchecked(y, x, color);
                    } else {
                        (*self_ptr.0).set_pixel_unchecked(x, y, color);
                    }
                })
        }
    }

    pub fn transform_screen_space_perspective(&self, point: &mut Vec3A) {
        let z_inverse = 1. / point.z();
        *point.x_mut() = (point.x() * z_inverse + 1.) * self.x_factor;
        *point.y_mut() = (-point.y() * z_inverse + 1.) * self.y_factor;
    }

    pub fn transform_screen_space_orthographic(&self, point: &mut Vec3A) {
        *point.x_mut() = (point.x() + 1.) * self.x_factor;
        *point.y_mut() = (-point.y() + 1.) * self.y_factor;
    }

    fn check_pixel_within_bounds(&self, pos: &Vec2) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    fn check_3d_pixel_within_bounds(&self, pos: &Vec3A) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub fn draw_indexed_parallel(&mut self, verts: &[Vec3A], indices: &[u32], color: &[u8; 4]) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            indices
                .par_chunks_exact(3)
                //.enumerate()
                .for_each(|tri| {
                    (*self_ptr.0).draw_triangle_parallel(
                        &[
                            *verts.get_unchecked(tri[0] as usize),
                            *verts.get_unchecked(tri[1] as usize),
                            *verts.get_unchecked(tri[2] as usize),
                        ],
                        color,
                    );
                })
        }
    }

    pub fn draw_indexed(&mut self, verts: &[Vec3A], indices: &[u32], color: &[u8; 4]) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            indices
                .par_chunks_exact(3)
                //.enumerate()
                .for_each(|tri| {
                    (*self_ptr.0).draw_triangle_fast(
                        &[
                            *verts.get_unchecked(tri[0] as usize),
                            *verts.get_unchecked(tri[1] as usize),
                            *verts.get_unchecked(tri[2] as usize),
                        ],
                        color,
                    );
                })
        }
    }
}

fn barycentric(points: &[Vec3A; 3], test: Vec2) -> Vec3A {
    let u = calc_barycentric(points, test);
    Vec3A::new(1. - (u.x() + u.y()) / u.z(), u.y() / u.z(), u.x() / u.z())
}

fn is_degenerate(points: &[Vec3A; 3], test: Vec2) -> bool {
    f32::abs(calc_barycentric(points, test).z()) < 1.
}

fn orient_2d(a: &Vec3A, b: &Vec3A, point: &Vec3A) -> f32 {
    (b.x() - a.x()) * (point.y() - a.y()) - (b.y() - a.y()) * (point.x() - a.x())
}

fn calc_barycentric(points: &[Vec3A; 3], test: Vec2) -> Vec3A {
    let v1 = Vec3A::new(
        points[2].x() - points[0].x(),
        points[1].x() - points[0].x(),
        points[0].x() - test.x(),
    );
    let v2 = Vec3A::new(
        points[2].y() - points[0].y(),
        points[1].y() - points[0].y(),
        points[0].y() - test.y(),
    );

    v1.cross(v2)
}

unsafe fn vec4_from_pixel_ptr(ptr: *const f32) -> Vec4 {
    use std::ptr::slice_from_raw_parts;;
    let data = slice_from_raw_parts(ptr, 4);
    vec4((*data)[0], (*data)[1], (*data)[2], (*data)[3])
}

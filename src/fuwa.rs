use super::handles::Handle;
use super::rasterization::Edge;
use super::HandleGenerator;
use super::Texture;
use crate::Triangle;
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
    pub pixels: Pixels<W>,
    depth_buffer: Vec<f32>,
    pub raster_par_count: usize,
    pub(crate) textures: HashMap<Handle<Texture>, Texture>,
    pub(crate) texture_generator: HandleGenerator<Texture>,
}

lazy_static! {
    static ref RENDER_MASK: Vec4 = Vec4::splat(0.);
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
pub(crate) struct FuwaPtr<W: HasRawWindowHandle>(pub(crate) *mut Fuwa<W>);

unsafe impl<W: HasRawWindowHandle> Send for FuwaPtr<W> {}
unsafe impl<W: HasRawWindowHandle> Sync for FuwaPtr<W> {}

impl<W: HasRawWindowHandle + Send + Sync> Fuwa<W> {
    pub(crate) fn get_self_ptr(&mut self) -> FuwaPtr<W> {
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

    pub(crate) fn pos_to_index(&self, x: u32, y: u32) -> usize {
        4 * (x + (y * self.width)) as usize
    }

    pub(crate) fn set_pixel_by_index(&mut self, index: usize, color: &[u8; 4]) {
        unsafe {
            self.pixels
                .get_frame()
                .get_unchecked_mut(index..index + 4)
                .copy_from_slice(color);
        }
    }

    pub(crate) fn draw_line(&mut self, mut start: Vec3A, mut end: Vec3A, color: &[u8; 4]) {
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
        let z_inverse = point.z().recip();
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

    pub fn draw_indexed_parallel(
        &mut self,
        verts: &[Vec3A],
        indices: &[u32],
        cull_flags: &[bool],
        color: &[u8; 4],
    ) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            indices
                .par_chunks_exact(3)
                .enumerate()
                .for_each(|(tri, index_list)| {
                    if !cull_flags[tri] {
                        (*self_ptr.0).draw_triangle_parallel(
                            &Triangle::from_data(verts, &index_list),
                            color,
                        );
                    }
                })
        }
    }

    pub fn draw_indexed(&mut self, verts: &[Vec3A], indices: &[u32], color: &[u8; 4]) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            indices
                .chunks_exact(3)
                //.par_chunks_exact(3)
                //.enumerate()
                .for_each(|index_list| {
                    (*self_ptr.0)
                        .draw_triangle_fast(&Triangle::from_data(verts, &index_list), color);
                })
        }
    }
}

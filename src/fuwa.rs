use super::FuwaStats;
use super::Texture;
use crate::{FragmentShaderFunction, Uniforms};
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
    pub(crate) uniforms: Uniforms,
    //fuwa_data: FuwaData,
    pub raster_par_count: usize,
}

lazy_static! {
    static ref THREAD_COLOR: [&'static [u8; 4]; 8] = [
        &super::colors::RED,
        &super::colors::GREEN,
        &super::colors::BLUE,
        &super::colors::CYAN,
        &super::colors::MAGENTA,
        &super::colors::YELLOW,
        &super::colors::PINK,
        &super::colors::OFFWHITE,
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
            uniforms: Uniforms::new(),
            //fuwa_data: FuwaData::new(),
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

    pub fn clear_color(&mut self, color: &[u8; 4]) {
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
            .par_chunks_mut(4 * 16)
            .for_each(|pixel_chunk| {
                pixel_chunk.copy_from_slice(&color_bytes);
            })
    }

    pub fn clear(&mut self) {
        self.pixels
            .get_frame()
            .par_iter_mut()
            .for_each(|pixel_chunk| {
                *pixel_chunk = 0;
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

    pub fn draw_box(&mut self, top_left: Vec3A, bottom_right: Vec3A, color: &[u8; 4]) {
        let top_right = vec3a(bottom_right.x(), top_left.y(), 0.);
        let bottom_left = vec3a(top_left.x(), bottom_right.y(), 0.);

        self.draw_line(top_left.clone(), top_right.clone(), color);
        self.draw_line(top_right.clone(), bottom_right.clone(), color);
        self.draw_line(bottom_right.clone(), bottom_left.clone(), color);
        self.draw_line(bottom_left.clone(), top_left.clone(), color);
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

    pub fn transform_screen_space_perspective(&self, vertex: &mut [f32], position_index: usize) {
        let z_inverse = vertex[position_index + 2].recip();

        for v in vertex.iter_mut() {
            *v *= z_inverse;
        }

        vertex[position_index] = (vertex[position_index] + 1.) * self.x_factor;
        vertex[position_index + 1] = (-vertex[position_index + 1] + 1.) * self.y_factor;
        vertex[position_index + 2] = z_inverse;
    }

    pub fn transform_screen_space_orthographic(&self, point: &mut [f32]) {
        point[0] = (point[0] + 1.) * self.x_factor;
        point[1] = (-point[1] + 1.) * self.y_factor;
    }

    fn check_3d_pixel_within_bounds(&self, pos: &Vec3A) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub(crate) fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        self.set_pixel_by_index(self.pos_to_index(x, y), color)
    }

    pub(crate) fn try_set_depth_block(
        &mut self,
        (block_x, block_y): (u32, u32),
        (width, height): (u32, u32),
        depths: Vec<f32>,
    ) -> Option<Vec<Option<f32>>> {
        let mut output = Vec::with_capacity((width * height) as usize);
        let mut idx = 0;

        unsafe {
            for y in block_y..block_y + height {
                let y_offset = y * self.width;
                let prev = self.depth_buffer.get_unchecked_mut(
                    (y_offset + block_x) as usize..(y_offset + block_x + width) as usize,
                );
                for prev_value in prev.iter_mut() {
                    if depths[idx] < *prev_value {
                        *prev_value = depths[idx];
                        output.push(Some(depths[idx].recip()));
                    } else {
                        output.push(None);
                    }
                    idx += 1;
                }
            }
        }

        if output.len() > 0 {
            Some(output)
        } else {
            None
        }
    }

    pub(crate) fn set_pixels_block(
        &mut self,
        (block_x, block_y): (u32, u32),
        (width, height): (u32, u32),
        depth_pass: &[Option<f32>],
        interp: Vec<Vec<f32>>,
        fragment_shader: &FragmentShaderFunction,
    ) {
        let mut idx = 0;
        for y in block_y..block_y + height {
            for x in block_x..block_x + width {
                if depth_pass[idx].is_some() {
                    self.set_pixel_unchecked(
                        x,
                        y,
                        &(fragment_shader)(&interp[idx], &self.uniforms),
                    );
                    idx += 1;
                }
            }
        }
    }

    pub fn load_texture(&mut self, path: String, set: u8, binding: u8) {
        let image_bytes = std::fs::read(format!("./resources/{}", &path)).unwrap();
        let image_data = image::load_from_memory(&image_bytes).unwrap();
        let image_data = image_data.as_rgba8().unwrap();
        let dimensions = image_data.dimensions();

        //use bincode::serialize;
        let texture = Texture {
            width: dimensions.0,
            height: dimensions.1,
            data: image_data.to_vec(),
        };

        //TODO: FIX THIS LATER
        self.uniforms.insert_texture(texture);
        // self.uniforms
        //     .insert(set, binding, serialize(&texture).unwrap())
    }
}

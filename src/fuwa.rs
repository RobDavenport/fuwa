use super::Texture;
use crate::{
    rasterization::{FragmentBuffer, FragmentKey, FragmentSlabMap, SlabPtr},
    render_pipeline::DepthBuffer,
    FSInput,
};
use crate::{FragmentShader, Uniforms};
use glam::*;
use image::GenericImageView;
use lazy_static::lazy_static;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;
use std::marker::{Send, Sync};
use wide::f32x8;

pub struct Fuwa<W: HasRawWindowHandle> {
    pub width: u32,
    pub height: u32,
    pub pixel_count: u32,
    pub x_factor: f32,
    pub y_factor: f32,
    pub pixels: Pixels<W>,
    pub(crate) depth_buffer: DepthBuffer,
    pub(crate) fragment_buffer: FragmentBuffer,
    pub fragment_slab_map: FragmentSlabMap,
    pub(crate) uniforms: Uniforms,
    pub(crate) thread_count: usize, //Do i need this?
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
        thread_count: usize,
        vsync: bool,
        high_performance: Option<bool>,
        window: &W,
    ) -> Self {
        Self {
            width,
            height,
            pixel_count: width * height,
            depth_buffer: DepthBuffer::new(width, height),
            thread_count,
            x_factor: width as f32 * 0.5,
            y_factor: height as f32 * 0.5,
            uniforms: Uniforms::new(),
            fragment_buffer: FragmentBuffer::new(width, height),
            fragment_slab_map: FragmentSlabMap::new(),
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

    pub fn clear_all(&mut self) {
        self.clear();
        self.depth_buffer.clear();
    }

    pub fn clear(&mut self) {
        //TODO: Is this faster than parallel?
        unsafe {
            let target = self.pixels.get_frame().as_mut_ptr();
            let len = self.pixels.get_frame().len();
            std::ptr::write_bytes(target, 0, len);
        }
        // let frame = self.pixels.get_frame();
        // let step = frame.len() / self.thread_count;
        // frame.par_chunks_mut(step).for_each(|pixel_chunk| unsafe {
        //     std::ptr::write_bytes(pixel_chunk.as_mut_ptr(), 0, pixel_chunk.len());
        // })
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.clear();
    }

    pub fn try_set_depth(&mut self, x: u32, y: u32, depth: f32) -> bool {
        self.depth_buffer
            .try_set_depth((x + y * self.width) as usize, depth)
    }

    pub fn try_set_depth_simd(&mut self, x: u32, y: u32, depths: &f32x8) -> Option<f32x8> {
        self.depth_buffer
            .try_set_depth_simd((x + y * self.width) as usize, depths)
    }

    pub(crate) fn set_fragment(&mut self, x: u32, y: u32, frag: FragmentKey) {
        self.fragment_buffer
            .set_fragment((x + y * self.width) as usize, frag);
    }

    pub fn render<F: FSInput>(&mut self, shader: &impl FragmentShader<F>, shader_index: usize) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            let slab = (*self_ptr.0).fragment_slab_map.get_mut_slab::<F>();
            (*self_ptr.0)
                .fragment_buffer
                .get_fragments_view_mut()
                .par_iter_mut()
                .enumerate()
                .filter(|(_index, fragment)| fragment.is_some())
                .for_each(|(index, fragment)| {
                    let frag = fragment.as_ref().unwrap();
                    if frag.shader_index == shader_index {
                        let color = shader.fragment_shader_fn(
                            slab.take(frag.fragment_key).unwrap(),
                            &(*self_ptr.0).uniforms,
                        );
                        (*self_ptr.0).set_pixel_by_index(index << 2, &color);
                        *fragment = None;
                    }
                });
        }
    }

    pub fn present(&mut self) -> Result<(), Error> {
        self.pixels.render()
    }

    //TODO: Use 0 for depth min sice we do 1/z
    pub fn render_depth_buffer(&mut self) -> Result<(), Error> {
        let pixel_iter = self.pixels.get_frame().par_chunks_exact_mut(4);
        let mut depth_max = f32::NEG_INFINITY;
        let mut depth_min = f32::INFINITY;

        self.depth_buffer.depth_buffer.iter().for_each(|x| {
            if x.is_finite() {
                if *x > depth_max {
                    depth_max = *x
                } else if *x < depth_min {
                    depth_min = *x
                }
            }
        });

        let range = depth_max - depth_min;
        let depth_iter = self.depth_buffer.depth_buffer.par_iter_mut();

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

        let depth_iter = self.depth_buffer.depth_buffer.par_iter_mut();
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

        self.draw_line(top_left, top_right, color);
        self.draw_line(top_right, bottom_right, color);
        self.draw_line(bottom_right, bottom_left, color);
        self.draw_line(bottom_left, top_left, color);
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

    pub fn transform_screen_space_perspective<F: FSInput>(
        &self,
        point: &mut Vec3A,
        interpolant: &mut F,
    ) {
        let z_inverse = point.z().recip();

        *interpolant *= z_inverse;

        *point.x_mut() = ((point.x() * z_inverse) + 1.) * self.x_factor;
        *point.y_mut() = ((-point.y() * z_inverse) + 1.) * self.y_factor;
        *point.z_mut() = z_inverse;
    }

    pub fn transform_screen_space_orthographic(&self, point: &mut Vec3A) {
        *point.x_mut() = (point.x() + 1.) * self.x_factor;
        *point.y_mut() = (-point.y() + 1.) * self.y_factor;
    }

    fn check_3d_pixel_within_bounds(&self, pos: &Vec3A) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub(crate) fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        //optick::event!();
        self.set_pixel_by_index(self.pos_to_index(x, y), color)
    }

    pub(crate) fn try_set_depth_block(
        &mut self,
        (block_x, block_y): (u32, u32),
        (width, height): (u32, u32),
        depths: Vec<f32>,
    ) -> Option<Vec<Option<f32>>> {
        //optick::event!();
        let mut output = Vec::with_capacity((width * height) as usize);
        let mut idx = 0;

        unsafe {
            for y in block_y..block_y + height {
                let y_offset = y * self.width;
                let prev = self.depth_buffer.depth_buffer.get_unchecked_mut(
                    (y_offset + block_x) as usize..(y_offset + block_x + width) as usize,
                );
                for prev_value in prev.iter_mut() {
                    if depths[idx] > *prev_value {
                        *prev_value = depths[idx];
                        output.push(Some(depths[idx]));
                    } else {
                        output.push(None);
                    }
                    idx += 1;
                }
            }
        }

        if !output.is_empty() {
            Some(output)
        } else {
            None
        }
    }

    pub(crate) fn set_fragments_block<F: FSInput>(
        &mut self,
        (block_x, block_y): (u32, u32),
        (block_width, block_height): (u32, u32),
        depth_pass: &[Option<f32>],
        interp: Vec<F>,
        fs_index: usize,
        slab_ptr: SlabPtr<F>,
    ) {
        let mut idx = 0;
        let mut interp = interp.into_iter();
        for y in block_y..block_y + block_height {
            for x in block_x..block_x + block_width {
                if depth_pass[idx].is_some() {
                    let frag = slab_ptr.insert_fragment(fs_index, interp.next().unwrap());
                    self.set_fragment(x, y, frag);
                }
                idx += 1;
            }
        }
    }

    pub(crate) fn set_fragments_simd<F: FSInput>(
        &mut self,
        pixel_x: u32,
        pixel_y: u32,
        interp: [F; 8],
        depth_pass: f32x8,
        fs_index: usize,
        slab_ptr: SlabPtr<F>,
    ) {
        let depth_pass = depth_pass.move_mask();
        for pixel in 0..8 {
            if 1 << pixel & depth_pass != 0 {
                let frag = slab_ptr.insert_fragment(fs_index, interp[pixel as usize]);
                self.set_fragment(pixel_x + pixel, pixel_y, frag)
            }
        }
    }

    // pub(crate) fn set_pixels_block(
    //     &mut self,
    //     (block_x, block_y): (u32, u32),
    //     (width, height): (u32, u32),
    //     depth_pass: &[Option<f32>],
    //     interp: Vec<Vec<f32>>,
    //     fragment_shader: &FragmentShaderFunction,
    // ) {
    //     //optick::event!();
    //     let mut idx = 0;
    //     for y in block_y..block_y + height {
    //         for x in block_x..block_x + width {
    //             if depth_pass[idx].is_some() {
    //                 self.set_pixel_unchecked(
    //                     x,
    //                     y,
    //                     &(fragment_shader)(&interp[idx], &self.uniforms),
    //                 );
    //                 idx += 1;
    //             }
    //         }
    //     }
    // }

    pub fn load_texture(&mut self, path: String) -> usize {
        let image_bytes = std::fs::read(format!("./resources/{}", &path)).unwrap();
        let image_data = image::load_from_memory(&image_bytes).unwrap();
        let dimensions = image_data.dimensions();
        let image_data = if let Some(data) = image_data.as_rgba8() {
            data.to_vec()
        } else {
            image_data.into_rgba().to_vec()
        };

        self.uniforms.add_texture(Texture {
            width: dimensions.0,
            height: dimensions.1,
            data: image_data.to_vec(),
        })
    }
}

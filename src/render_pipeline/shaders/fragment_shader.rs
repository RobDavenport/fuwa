use crate::{Texture, Uniforms};
use glam::*;
use lazy_static::lazy_static;
use std::ops::*;

lazy_static! {
    static ref U8FASTVEC: Vec3A = Vec3A::splat(255.);
}

pub trait FSInput:
    Clone
    + Copy
    + Sized
    + Send
    + Sync
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Mul<f32, Output = Self>
    + MulAssign<f32>
    + Div<f32, Output = Self>
    + DivAssign<f32>
{
}
impl FSInput for Vec3A {}
impl FSInput for Vec2 {}

pub trait FragmentShader<F: FSInput>: Send + Sync + Clone {
    fn fragment_shader_fn(&self, fs_in: F, uniforms: &Uniforms) -> [u8; 4];
}

#[derive(Clone)]
pub struct ColorBlend {}
impl ColorBlend {
    pub fn new() -> Self {
        Self {}
    }
}
impl FragmentShader<Vec3A> for ColorBlend {
    fn fragment_shader_fn(&self, fs_in: Vec3A, _: &Uniforms) -> [u8; 4] {
        let c = fs_in * *U8FASTVEC;
        [c[0] as u8, c[1] as u8, c[2] as u8, 0xFF]
    }
}

#[derive(Clone)]
pub struct Textured {
    texture_handle: usize,
}

impl Textured {
    pub fn new(texture_handle: usize) -> Self {
        Self { texture_handle }
    }

    pub fn set_texture_handle(&mut self, texture_handle: usize) {
        self.texture_handle = texture_handle
    }

    pub fn get_texture_handle(&self) -> usize {
        self.texture_handle
    }
}

impl FragmentShader<Vec2> for Textured {
    fn fragment_shader_fn(&self, fs_in: Vec2, uniforms: &Uniforms) -> [u8; 4] {
        sample_2d(
            fs_in[0],
            fs_in[1],
            uniforms.get_texture(self.texture_handle),
        )
    }
}

fn sample_2d(u: f32, v: f32, texture: &Texture) -> [u8; 4] {
    let u = (u * (texture.width - 1) as f32).round() as u32;
    let v = (v * (texture.height - 1) as f32).round() as u32;
    let index = 4 * (u + (v * texture.width)) as usize;
    [
        texture.data[index],
        texture.data[index + 1],
        texture.data[index + 2],
        texture.data[index + 3],
    ]
}

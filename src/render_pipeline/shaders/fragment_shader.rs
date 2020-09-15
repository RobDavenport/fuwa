use crate::{Texture, Uniforms};
use glam::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref U8FASTVEC: Vec3A = Vec3A::splat(255.);
}

pub type FragmentShaderFunction = Box<dyn Fn(&[f32], &Uniforms) -> [u8; 4] + Send + Sync>;

pub struct FragmentShader {
    pub fragment_shader: FragmentShaderFunction,
}

impl FragmentShader {
    pub fn color_blend() -> Self {
        Self {
            fragment_shader: Box::new(|data, _| {
                let c = Vec3A::from([data[3], data[4], data[5]]) * *U8FASTVEC;
                [c[0] as u8, c[1] as u8, c[2] as u8, 0xFF]
            }),
        }
    }

    pub fn textured(_set: u8, _binding: u8) -> Self {
        Self {
            fragment_shader: Box::new(move |data, uniforms| {
                //sample_2d(data[3], data[4], uniforms.get::<Texture>(&set, &binding))
                sample_2d(data[3], data[4], uniforms.get_texture())
            }),
        }
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

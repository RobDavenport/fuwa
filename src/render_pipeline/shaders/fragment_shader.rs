use crate::{Texture, Uniforms};
use glam::*;

pub type FragmentShaderFunction = Box<dyn Fn(&[f32], &Uniforms) -> [u8; 4] + Send + Sync>;

pub struct FragmentShader {
    pub fragment_shader: FragmentShaderFunction,
}

impl FragmentShader {
    pub fn color_blend() -> Self {
        Self {
            fragment_shader: Box::new(|data, _| {
                [
                    (data[3] * 255.) as u8,
                    (data[4] * 255.) as u8,
                    0x00, //(data[5] * 255.) as u8,
                    0xFF,
                ]
            }),
        }
    }

    pub fn textured(set: u8, binding: u8) -> Self {
        Self {
            fragment_shader: Box::new(move |data, uniforms| {
                //sample_2d(data[3], data[4], uniforms.get::<Texture>(&set, &binding))
                sample_2d(data[3], data[4], uniforms.get_texture())
            }),
        }
    }
}

fn sample_2d(u: f32, v: f32, texture: &Texture) -> [u8; 4] {
    let u = u * texture.width as f32;
    let v = v * texture.height as f32;
    let u = (u as u32).min(texture.width);
    let v = (v as u32).min(texture.height);
    let index = (((u + v * texture.width) * 4)).min((texture.width * texture.height * 4) - 1) as usize;
    [
        texture.data[index],
        texture.data[index + 1],
        texture.data[index + 2],
        texture.data[index + 3],
    ]
}

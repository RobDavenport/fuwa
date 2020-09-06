use glam::*;

pub struct FragmentShader {
    pub fragment_shader: fn(&[f32]) -> [u8; 4],
}

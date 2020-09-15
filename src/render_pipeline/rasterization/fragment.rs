use crate::{FragmentShaderFunction, Uniforms};

#[derive(Clone)]
pub(crate) struct Fragment<'fs> {
    pub(crate) interpolants: Vec<f32>,
    pub(crate) shader: &'fs FragmentShaderFunction,
}

impl<'fs> Fragment<'fs> {
    pub(crate) fn run(&self, uniforms: &Uniforms) -> [u8; 4] {
        (self.shader)(&self.interpolants, uniforms)
    }
}

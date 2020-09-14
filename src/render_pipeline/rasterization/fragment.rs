use crate::{FragmentShaderFunction, Uniforms};

#[derive(Clone)]
pub(crate) struct Fragment<'a> {
    pub(crate) interpolants: Vec<f32>,
    pub(crate) shader: &'a FragmentShaderFunction,
}

impl<'a> Fragment<'a> {
    pub(crate) fn run(&self, uniforms: &Uniforms) -> [u8; 4] {
        (self.shader)(&self.interpolants, uniforms)
    }
}

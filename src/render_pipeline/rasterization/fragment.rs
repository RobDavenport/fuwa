// use crate::FSInput;
// use crate::{FragmentShader, Uniforms};

// #[derive(Clone)]
// pub(crate) struct Fragment<F: FSInput, S: FragmentShader<F>> {
//     pub(crate) interpolants: F,
//     pub(crate) shader: S,
// }

// impl<F: FSInput, S: FragmentShader<F>> Fragment<F, S> {
//     pub(crate) fn run(&self, uniforms: &Uniforms) -> [u8; 4] {
//         self.shader.fragment_shader_fn(self.interpolants, uniforms)
//     }
// }

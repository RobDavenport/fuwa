//TODO: Write this struct
use glam::*;

pub trait VSInput: Send + Sync {}
impl VSInput for [f32; 5] {}
impl VSInput for [f32; 6] {}

//pub type VertexShaderFunction<VSIn, VSOut> = Box<dyn Fn(VSIn) -> (Vec3A, VSOut) + Send + Sync>;

pub trait VertexShader<VSIn, VSOut>: Send + Sync {
    fn vertex_shader_fn(&self, raw_vertex_data: &VSIn) -> (Vec3A, VSOut);
}

pub struct BasicVertexShader {
    rotation: Mat3,
    translation: Vec3A,
}

impl BasicVertexShader {
    pub fn bind_rotation(&mut self, rotation: Mat3) {
        self.rotation = rotation;
    }

    pub fn bind_translation(&mut self, translation: Vec3A) {
        self.translation = translation;
    }

    pub fn new() -> Self {
        Self {
            rotation: Mat3::default(),
            translation: Vec3A::default(),
        }
    }
}

impl VertexShader<[f32; 5], Vec2> for BasicVertexShader {
    fn vertex_shader_fn(&self, raw_vertex_data: &[f32; 5]) -> (Vec3A, Vec2) {
        let position = self.rotation
            * vec3a(raw_vertex_data[0], raw_vertex_data[1], raw_vertex_data[2])
            + self.translation;
        let output = vec2(raw_vertex_data[3], raw_vertex_data[4]);

        (position, output)
    }
}

impl VertexShader<[f32; 6], Vec3A> for BasicVertexShader {
    fn vertex_shader_fn(&self, raw_vertex_data: &[f32; 6]) -> (Vec3A, Vec3A) {
        let position = self.rotation
            * vec3a(raw_vertex_data[0], raw_vertex_data[1], raw_vertex_data[2])
            + self.translation;
        let output = vec3a(raw_vertex_data[3], raw_vertex_data[4], raw_vertex_data[5]);

        (position, output)
    }
}

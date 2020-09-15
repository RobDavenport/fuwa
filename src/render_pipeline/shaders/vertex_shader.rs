//TODO: Write this struct
pub type VertexShaderFunction<VSIn, VSOut> = Box<dyn Fn(VSIn) -> VSOut + Send + Sync>;

pub struct VertexShader<VSIn, VSOut> {
    pub vertex_shader: VertexShaderFunction<VSIn, VSOut>,
}

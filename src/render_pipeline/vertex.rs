use glam::*;

pub struct VertexData<'a>(pub &'a mut [f32]);

impl<'a> VertexData<'a> {
    pub fn get_position(&self, position_index: usize) -> Vec3A {
        vec3a(
            self.0[position_index],
            self.0[position_index + 1],
            self.0[position_index + 2],
        )
    }
}

// pub trait Vertex {
//   fn position_mut(&mut self) -> &mut Vec3A;
//   fn position(&self) -> Vec3A;
// }

// pub(crate) struct VertexColor {
//   pub(crate) pos: Vec3A,
//   pub(crate) color: Vec3A,
// }

// impl Vertex for VertexColor {
//   fn position_mut(&mut self) -> &mut Vec3A {
//     &mut self.pos
//   }

//   fn position(&self) -> Vec3A {
//     self.pos
//   }
// }

// pub(crate) struct VertexTexture {
//   pub(crate) pos: Vec3A,
//   pub(crate) uv: Vec2,
// }

// impl Vertex for VertexTexture {
//   fn position_mut(&mut self) -> &mut Vec3A {
//     &mut self.pos
//   }

//   fn position(&self) -> Vec3A {
//     self.pos
//   }
// }

// pub(crate) struct VertexTextureNormal {
//   pub(crate) pos: Vec3A,
//   pub(crate) uv: Vec2,
//   pub(crate) normal: Vec3A,
// }

// impl Vertex for VertexTextureNormal {
//   fn position_mut(&mut self) -> &mut Vec3A {
//     &mut self.pos
//   }

//   fn position(&self) -> Vec3A {
//     self.pos
//   }
// }

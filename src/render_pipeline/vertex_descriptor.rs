pub struct VertexDescriptor {
    //fields: Vec<VertexDescriptorField>,
    pub(crate) stride: usize,
    pub(crate) position_index: usize,
}

pub enum VertexDescriptorField {
    Vec2,
    Vec3,
    Vec4,
}

impl VertexDescriptor {
    pub fn new(fields: Vec<VertexDescriptorField>, position_index: usize) -> Self {
        let stride = fields.iter().fold(0, |a, b| a + b.size());
        Self {
            //fields,
            stride,
            position_index,
        }
    }
}

impl VertexDescriptorField {
    fn size(&self) -> usize {
        use VertexDescriptorField::*;

        match self {
            Vec2 => 2,
            Vec3 => 3,
            Vec4 => 4,
        }
    }
}

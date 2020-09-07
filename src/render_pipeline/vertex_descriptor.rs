use glam::*;

pub struct VertexDescriptor {
    pub(crate) fields: Vec<VertexDescriptorField>,
    pub(crate) stride: usize,
    pub(crate) position_index: usize,
    //pub(crate) position_descriptor: VertexPositionDescriptor,
}

pub enum VertexDescriptorField {
    Vec2,
    Vec3,
    Vec4,
}

// pub struct VertexPositionDescriptor {
//     pub position: usize,
//     pub field_type: VertexDescriptorField,
// }

impl VertexDescriptor {
    pub fn new(
        fields: Vec<VertexDescriptorField>,
        position_index: usize,
        //position_descriptor: VertexPositionDescriptor,
    ) -> Self {
        let stride = fields.iter().fold(0, |a, b| a + b.size());
        Self {
            fields,
            stride,
            position_index,
            //position_descriptor,
        }
    }

    // pub(crate) fn get_position_range(&self) -> (usize, usize) {
    //   let start = (0..self.position_descriptor.position)
    //   .fold(0, |acc, idx| acc + self.fields[idx].size());

    //   (start, start + self.position_descriptor.field_type.size())
    // }

    pub(crate) fn get_position_range(&self) -> usize {
        (0..self.position_index).fold(0, |acc, idx| acc + self.fields[idx].size())
    }

    // pub(crate) fn copy_positions(&self, vertex_list: &[f32]) -> Vec<Vec3A> {
    //     use VertexDescriptorField as VD;

    //     let range_start = (0..self.position_descriptor.position)
    //         .fold(0, |acc, idx| acc + self.fields[idx].size());

    //     match self.position_descriptor.field_type {
    //         VD::Vec2 => Self::handle_vec2(vertex_list, range_start, self.stride),
    //         VD::Vec3 => Self::handle_vec3(vertex_list, range_start, self.stride),
    //         VD::Vec4 => panic!("NOT IMPLEMENTED!"),
    //     }
    // }

    // fn handle_vec2(vertex_list: &[f32], start: usize, stride: usize) -> Vec<Vec3A> {
    //     vertex_list
    //         .chunks_exact(stride)
    //         .map(|chunk| vec3a(chunk[start], chunk[start + 1], 0.))
    //         .collect()
    // }

    // fn handle_vec3(vertex_list: &[f32], start: usize, stride: usize) -> Vec<Vec3A> {
    //     vertex_list
    //         .chunks_exact(stride)
    //         .map(|chunk| vec3a(chunk[start], chunk[start + 1], chunk[start + 2]))
    //         .collect()
    // }
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

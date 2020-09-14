use rayon::prelude::*;
pub(crate) struct DepthBuffer {
    pub(crate) depth_buffer: Vec<f32>,
    //TODO: Implement depth functions like GREATER THAN or LESS THAN
}

impl DepthBuffer {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            depth_buffer: vec![f32::INFINITY; (width * height) as usize],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.depth_buffer.par_iter_mut().for_each(|x| {
            *x = f32::INFINITY;
        });
    }

    pub fn try_set_depth(&mut self, index: usize, depth: f32) -> bool {
        //optick::event!();

        unsafe {
            let prev = self.depth_buffer.get_unchecked_mut(index);
            if depth < *prev {
                *prev = depth;
                true
            } else {
                false
            }
        }
    }

    //TODO: Move Try_SET_DEPTH_BLOCK to here
}

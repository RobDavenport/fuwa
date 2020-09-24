use bytemuck::cast;
use rayon::prelude::*;
use wide::f32x8;
pub(crate) struct DepthBuffer {
    pub(crate) depth_buffer: Vec<f32>,
    //TODO: Implement depth functions like GREATER THAN or LESS THAN
}

impl DepthBuffer {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            depth_buffer: vec![f32::NEG_INFINITY; (width * height) as usize],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.depth_buffer
            .par_iter_mut()
            .filter(|x| **x != f32::NEG_INFINITY)
            .for_each(|x| {
                *x = f32::NEG_INFINITY;
            });
    }

    pub fn try_set_depth(&mut self, index: usize, depth: f32) -> bool {
        //optick::event!();

        unsafe {
            let prev = self.depth_buffer.get_unchecked_mut(index);
            if depth > *prev {
                *prev = depth;
                true
            } else {
                false
            }
        }
    }

    pub fn try_set_depth_simd(&mut self, index: usize, depths: &f32x8) -> Option<f32x8> {
        unsafe {
            let prev = f32x8::from(self.depth_buffer.get_unchecked(index..index + 8));
            let depth_pass_mask = depths.cmp_gt(prev);

            if depth_pass_mask.any() {
                self.depth_buffer
                    .get_unchecked_mut(index..index + 8)
                    .copy_from_slice(&cast::<_, [f32; 8]>(depths.max(prev)));
                Some(depth_pass_mask)
            } else {
                None
            }
        }
    }

    //TODO: Move Try_SET_DEPTH_BLOCK to here
}

use bytemuck::cast;
//use rayon::prelude::*;
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
        //TODO: Is this safe/faster than parallel?
        unsafe {
            let target = self.depth_buffer.as_mut_ptr();
            let len = self.depth_buffer.len();
            std::ptr::write_bytes(target, 0, len)
        }
        //let step = self.depth_buffer.len() / self.thread_count;
        //self.depth_buffer
        // .par_chunks_mut(step)
        // .for_each(|row| unsafe {
        //     //TODO: Is this safe? investigate
        //     std::ptr::write_bytes(row.as_mut_ptr(), 0, row.len());

        //     //for pixel in row.into_iter() {
        //     //if *pixel != f32::NEG_INFINITY {
        //     //    std::ptr::write(pixel, f32::NEG_INFINITY);
        //     //}
        //     //}
        // });
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

use super::Fragment;
use rayon::prelude::*;

pub(crate) struct FragmentBuffer<'f> {
    fragment_buffer: Vec<Option<Fragment<'f>>>,
}

impl<'f> FragmentBuffer<'f> {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            fragment_buffer: vec![None; (width * height) as usize],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.fragment_buffer.par_iter_mut().for_each(|f| *f = None);
    }

    pub(crate) fn set_fragment(&mut self, index: usize, fragment: Fragment<'f>) {
        self.fragment_buffer[index] = Some(fragment);
    }

    pub(crate) fn get_fragments_view(&self) -> &[Option<Fragment>] {
        &self.fragment_buffer
    }
}

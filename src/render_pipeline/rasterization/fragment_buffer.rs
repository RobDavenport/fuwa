use super::Fragment;
use crate::{FSInput, FragmentShader};
use rayon::prelude::*;

pub(crate) struct FragmentBuffer<F: FSInput, S: FragmentShader<F>> {
    fragment_buffer: Vec<Option<Fragment<F, S>>>,
}

impl<F: FSInput, S: FragmentShader<F>> FragmentBuffer<F, S> {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            fragment_buffer: vec![None; (width * height) as usize],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.fragment_buffer
            .par_iter_mut()
            .filter(|x| x.is_some())
            .for_each(|f| *f = None);
    }

    pub(crate) fn set_fragment(&mut self, index: usize, fragment: Fragment<F, S>) {
        self.fragment_buffer[index] = Some(fragment);
    }

    pub(crate) fn get_fragments_view(&self) -> &[Option<Fragment<F, S>>] {
        &self.fragment_buffer
    }
}

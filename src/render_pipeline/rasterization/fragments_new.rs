use crate::FSInput;
use slab::Slab;
use type_map::TypeMap;

pub(crate) struct FragmentBufferNew {
    fragments: Vec<Option<FragmentKey>>,
}

impl FragmentBufferNew {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            fragments: vec![None; (width * height) as usize],
        }
    }

    pub(crate) fn set_fragment(&mut self, index: usize, fragment: FragmentKey) {
        self.fragments[index] = Some(fragment)
    }

    pub(crate) fn get_fragments_view_mut(&mut self) -> &mut[Option<FragmentKey>] {
        &mut self.fragments
    }
}

#[derive(Clone)]
pub(crate) struct FragmentKey {
    pub(crate) shader_index: usize,
    pub(crate) fragment_key: usize,
}

pub(crate) struct FragmentSlabMap {
    slab_map: TypeMap,
}

impl FragmentSlabMap {
    pub(crate) fn new() -> Self {
        Self {
            slab_map: TypeMap::new()
        }
    }

    pub(crate) fn insert_fragment<F: FSInput + 'static>(
        &mut self,
        shader_index: usize,
        input: F,
    ) -> FragmentKey {
        if let Some(slab) = self.slab_map.get_mut::<Slab<F>>() {
            let fragment_key = slab.insert(input);
            FragmentKey {
                shader_index,
                fragment_key,
            }
        } else {
            let mut slab = Slab::new();
            let fragment_key = slab.insert(input);
            self.slab_map.insert::<Slab<F>>(slab);

            FragmentKey {
                shader_index,
                fragment_key,
            }
        }
    }

    pub(crate) fn get_slab<F: FSInput + 'static>(&self) -> &Slab<F> {
        self.slab_map.get::<Slab<F>>().unwrap()
    }

    pub(crate) fn remove_slab<F: FSInput + 'static>(&mut self) -> Slab<F> {
        self.slab_map.remove::<Slab<F>>().unwrap()
    }
    pub(crate) fn insert_slab<F: FSInput + 'static>(&mut self, slab: Slab<F>) {
        self.slab_map.insert(slab);
    }
}

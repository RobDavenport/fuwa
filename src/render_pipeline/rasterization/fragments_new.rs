use crate::FSInput;
use parking_lot::RwLock;
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

    pub(crate) fn get_fragments_view_mut(&mut self) -> &mut [Option<FragmentKey>] {
        &mut self.fragments
    }
}

#[derive(Clone)]
pub(crate) struct FragmentKey {
    pub(crate) shader_index: usize,
    pub(crate) fragment_key: usize,
}

pub(crate) struct FragmentSlabMap {
    slab_map: RwLock<TypeMap>,
}

impl FragmentSlabMap {
    pub(crate) fn new() -> Self {
        Self {
            slab_map: RwLock::new(TypeMap::new()),
        }
    }

    pub(crate) fn insert_fragment<F: FSInput + 'static>(
        &mut self,
        shader_index: usize,
        input: F,
    ) -> FragmentKey {
        let mut write = self.slab_map.write();
        if let Some(slab) = write.get_mut::<Slab<F>>() {
            let fragment_key = slab.insert(input);
            FragmentKey {
                shader_index,
                fragment_key,
            }
        } else {
            let mut slab = Slab::new();
            let fragment_key = slab.insert(input);
            write.insert(slab);

            FragmentKey {
                shader_index,
                fragment_key,
            }
        }
    }

    // pub(crate) fn get_slab<F: FSInput + 'static>(&self) -> Option<&Slab<F>> {
    //     self.slab_map.read().get::<Slab<F>>()
    // }

    // pub(crate) fn get_slab_mut<F: FSInput + 'static>(&mut self) -> Option<&mut Slab<F>> {
    //     self.slab_map.write().get_mut::<Slab<F>>()
    // }

    pub(crate) fn remove_slab<F: FSInput + 'static>(&mut self) -> Option<Slab<F>> {
        self.slab_map.write().remove::<Slab<F>>()
    }
    pub(crate) fn insert_slab<F: FSInput + 'static>(&mut self, slab: Slab<F>) {
        self.slab_map.write().insert(slab);
    }
}

unsafe impl<F> Send for SlabPtr<F> {}
unsafe impl<F> Sync for SlabPtr<F> {}
pub(crate) struct SlabPtr<F>(pub(crate) *mut Slab<F>);

impl<F> SlabPtr<F> {
    pub(crate) fn new(slab: &mut Slab<F>) -> Self {
        Self(slab as *mut Slab<F>)
    }
}

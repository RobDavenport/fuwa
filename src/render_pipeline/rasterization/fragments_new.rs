use crate::FSInput;
use sharded_slab::Slab;
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

pub struct FragmentSlabMap {
    slab_map: TypeMap,
}

impl FragmentSlabMap {
    pub(crate) fn new() -> Self {
        Self {
            slab_map: TypeMap::new(),
        }
    }

    pub fn get_mut_slab<F: FSInput + 'static>(&mut self) -> &mut Slab<F> {
        if !self.slab_map.contains::<Slab<F>>() {
            let slab: Slab<F> = Slab::new_with_config();
            self.slab_map.insert(slab);
        }

        self.slab_map.get_mut::<Slab<F>>().unwrap()
    }

    pub(crate) fn remove_slab<F: FSInput + 'static>(&mut self) -> Option<Slab<F>> {
        self.slab_map.remove::<Slab<F>>()
    }
    pub(crate) fn insert_slab<F: FSInput + 'static>(&mut self, slab: Slab<F>) {
        self.slab_map.insert(slab);
    }
}

unsafe impl<F> Send for SlabPtr<F> {}
unsafe impl<F> Sync for SlabPtr<F> {}
#[derive(Copy, Clone)]
pub(crate) struct SlabPtr<F>(pub(crate) *mut Slab<F>);

impl<F> SlabPtr<F> {
    pub(crate) fn new(slab: &mut Slab<F>) -> Self {
        Self(slab as *mut Slab<F>)
    }

    pub(crate) fn insert_fragment(&self, shader_index: usize, input: F) -> FragmentKey {
        unsafe {
            let fragment_key = (*self.0).insert(input).unwrap();
            FragmentKey {
                shader_index,
                fragment_key,
            }
        }
    }
}

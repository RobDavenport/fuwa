use crate::FSInput;
use dashmap::DashMap;
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
    data_map: TypeMap,
}

impl FragmentSlabMap {
    pub(crate) fn new() -> Self {
        Self {
            data_map: TypeMap::new(),
        }
    }

    pub fn get_mut_map<F: FSInput + 'static>(&mut self) -> &mut DashMap<usize, F> {
        if !self.data_map.contains::<DashMap<usize, F>>() {
            let map: DashMap<usize, F> = DashMap::new();
            self.data_map.insert(map);
        }

        self.data_map.get_mut::<DashMap<usize, F>>().unwrap()
    }
}

unsafe impl<F> Send for MapPtr<F> {}
unsafe impl<F> Sync for MapPtr<F> {}
#[derive(Copy, Clone)]
pub(crate) struct MapPtr<F>(pub(crate) *mut DashMap<usize, F>);

impl<F> MapPtr<F> {
    pub(crate) fn new(map: &mut DashMap<usize, F>) -> Self {
        Self(map as *mut DashMap<usize, F>)
    }

    pub(crate) fn insert_fragment(&self, index: usize, input: F) {
        unsafe {
            (*self.0).insert(index, input);
        }
    }
}

use crate::Texture;
use slab::Slab;

pub struct Uniforms {
    textures: Slab<Texture>,
}

impl Uniforms {
    pub(crate) fn new() -> Self {
        Self {
            textures: Slab::new(),
        }
    }

    pub fn get_texture(&self, handle: usize) -> &Texture {
        unsafe { self.textures.get_unchecked(handle) }
    }

    pub fn add_texture(&mut self, texture: Texture) -> usize {
        self.textures.insert(texture)
    }
}

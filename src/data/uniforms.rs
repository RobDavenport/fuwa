use hashbrown::HashMap;

use crate::Texture;

//Set -> Binding
pub struct Uniforms {
    data: HashMap<u8, HashMap<u8, Vec<u8>>>,
    texture: Texture,
}

impl Uniforms {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
            texture: Texture {
                width: 0,
                height: 0,
                data: Vec::new(),
            },
        }
    }

    // pub fn get<'a, T: Deserialize<'a>>(&'a self, set: &u8, binding: &u8) -> T {
    //     let data = self.data.get(set).unwrap().get(binding).unwrap();
    //     deserialize::<T>(data).unwrap()
    // }
    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn insert_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }

    pub(crate) fn insert(&mut self, set: u8, binding: u8, data: Vec<u8>) {
        if let Some(found_set) = self.data.get_mut(&set) {
            found_set.insert(binding, data);
        } else {
            let mut new_set = HashMap::with_capacity(1);
            new_set.insert(binding, data);
            self.data.insert(set, new_set);
        };
    }
}

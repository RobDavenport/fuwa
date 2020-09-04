use crate::fuwa;

pub enum TextureFormat {
    ARGB,
    RGB,
    RGBA,
}

pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(data: Vec<u8>, format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    fn get_data(&self) -> &[u8] {
        &self.data
    }
}

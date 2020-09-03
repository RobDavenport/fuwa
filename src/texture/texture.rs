use crate::fuwa;

pub enum TextureFormat {
    RGB,
    RGBA,
    ARGB,
}

pub struct Texture {
    data: Vec<u8>,
    pub format: TextureFormat,
    width: u32,
    height: u32,
}

impl Texture {
    fn new(data: Vec<u8>, format: TextureFormat, width: u32, height: u32) -> Self {
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

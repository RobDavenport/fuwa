// pub enum TextureFormat {
//     ARGB,
//     RGB,
//     RGBA,
// }

pub struct Texture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data: Vec<u8>,
}

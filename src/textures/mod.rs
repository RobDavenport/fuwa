// use crate::Fuwa;
// use crate::Handle;

mod texture;
pub use texture::*;

// use raw_window_handle::HasRawWindowHandle;

// impl<W: HasRawWindowHandle> Fuwa<W> {
//     pub fn upload_texture(&mut self, texture: Texture) -> Handle<Texture> {
//         let next = self.texture_generator.next_handle();
//         self.textures.insert(next, texture);
//         next
//     }

//     pub fn insert_texture(&mut self, handle: Handle<Texture>, texture: Texture) {
//         self.textures.insert(handle, texture);
//     }
// }

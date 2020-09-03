mod fuwa;
pub use fuwa::*;

mod rasterizer;
pub use rasterizer::*;

mod texture;
pub use texture::*;

mod handle;
pub use handle::*;

use glam::*;

pub mod Colors {
    //R, G, B, A
    pub const WHITE: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
    pub const LIGHTGRAY: [u8; 4] = [0xC8, 0xC8, 0xC8, 0xFF];
    pub const DARKGREY: [u8; 4] = [0x50, 0x50, 0x50, 0x50];
    pub const BLACK: [u8; 4] = [0x00, 0x00, 0x00, 0xFF];

    pub const RED: [u8; 4] = [0xFF, 0x00, 0x00, 0xFF];
    pub const GREEN: [u8; 4] = [0x00, 0xFF, 0x00, 0xFF];
    pub const BLUE: [u8; 4] = [0x00, 0x00, 0xFF, 0xFF];

    pub const CYAN: [u8; 4] = [0x00, 0xFF, 0xFF, 0xFF];
    pub const MAGENTA: [u8; 4] = [0xFF, 0x00, 0xFF, 0xFF];
    pub const YELLOW: [u8; 4] = [0xFF, 0xFF, 0x00, 0xFF];

    pub const PINK: [u8; 4] = [0xFF, 0x77, 0xA8, 0xFF];
    pub const PEACH: [u8; 4] = [0xFF, 0xCC, 0xAA, 0xFF];
    pub const OFFWHITE: [u8; 4] = [0xFF, 0xF1, 0xE8, 0xFF];
}

pub fn cube(size: f32) -> [Vec3A; 8] {
    let side = size * 0.5;
    [
        Vec3A::new(-side, -side, -side),
        Vec3A::new(side, -side, -side),
        Vec3A::new(-side, side, -side),
        Vec3A::new(side, side, -side),
        Vec3A::new(-side, -side, side),
        Vec3A::new(side, -side, side),
        Vec3A::new(-side, side, side),
        Vec3A::new(side, side, side),
    ]
}

pub fn tri(size: f32) -> [Vec3A; 3] {
    let side = size * 0.5;
    [
        Vec3A::new(-side, -side, 0.),
        Vec3A::new(side, -side, 0.),
        Vec3A::new(-side, side, 0.),
    ]
}

pub fn plane(size: f32) -> [Vec3A; 4] {
    let side = size * 0.5;
    [
        Vec3A::new(-side, -side, 0.),
        Vec3A::new(side, -side, 0.),
        Vec3A::new(-side, side, 0.),
        Vec3A::new(side, side, 0.),
    ]
}

pub fn tri_indices() -> [u32; 3] {
    [0, 1, 2]
}

pub fn plane_indices() -> [u32; 6] {
    [0, 1, 2, 2, 1, 3]
}

pub fn cube_lines() -> [u32; 24] {
    [
        0, 1, 1, 3, 3, 2, 2, 0, 0, 4, 1, 5, 3, 7, 2, 6, 4, 5, 5, 7, 7, 6, 6, 4,
    ]
}

pub fn cube_indices() -> [u32; 36] {
    [
        0, 1, 2, 2, 1, 3, 1, 5, 3, 3, 5, 7, 2, 3, 6, 3, 7, 6, 4, 7, 5, 4, 6, 7, 0, 2, 4, 2, 6, 4,
        0, 4, 1, 1, 4, 5,
    ]
}

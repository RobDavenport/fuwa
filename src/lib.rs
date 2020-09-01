mod fuwa;
pub use fuwa::*;

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

#[derive(Copy, Clone)]
struct Edge {
    one_step_x: Vec4,
    one_step_y: Vec4,
}

impl Edge {
    const STEP_X: usize = 4;
    const STEP_Y: usize = 1;

    pub fn init(v0: &Vec3A, v1: &Vec3A, origin: &Vec3A) -> (Self, Vec4) {
        let a = v0.y() - v1.y();
        let b = v1.x() - v0.x();
        let c = (v0.x() * v1.y()) - (v0.y() * v1.x());

        let origin_x = origin.x();
        let origin_y = origin.y();
        let x = vec4(origin_x, origin_x, origin_x, origin_x) + vec4(0., 1., 2., 3.);
        let y = vec4(origin_y, origin_y, origin_y, origin_y);

        (
            Self {
                one_step_x: vec4(a, a, a, a) * Self::STEP_X as f32,
                one_step_y: vec4(b, b, b, b) * Self::STEP_Y as f32,
            },
            (vec4(a, a, a, a) * x) + (vec4(b, b, b, b) * y) + vec4(c, c, c, c),
        )
    }
}

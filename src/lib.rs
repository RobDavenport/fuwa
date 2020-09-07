mod fuwa;
pub use fuwa::*;

mod textures;
pub use textures::*;

mod handles;
pub use handles::*;

mod render_pipeline;
pub use render_pipeline::*;

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

pub fn vec3_into_float_slice(vec: &[Vec3]) -> Vec<f32> {
    let mut output = Vec::<f32>::with_capacity(vec.len() * 3);

    vec.iter().for_each(|v| {
        output.push(v.x());
        output.push(v.y());
        output.push(v.z());
    });

    output
}

pub fn cube(size: f32) -> [Vec3; 8] {
    let size = size * 0.5;
    [
        vec3(-size, -size, -size),
        vec3(size, -size, -size),
        vec3(-size, size, -size),
        vec3(size, size, -size),
        vec3(-size, -size, size),
        vec3(size, -size, size),
        vec3(-size, size, size),
        vec3(size, size, size),
    ]
}

pub fn tri(size: f32) -> [Vec3; 3] {
    let size = size * 0.5;
    [
        vec3(-size, -size, 0.),
        vec3(size, -size, 0.),
        vec3(-size, size, 0.),
    ]
}

pub fn plane(size: f32) -> [Vec3; 4] {
    let size = size * 0.5;
    [
        vec3(-size, -size, 0.),
        vec3(size, -size, 0.),
        vec3(-size, size, 0.),
        vec3(size, size, 0.),
    ]
}

pub fn tri_indices() -> [usize; 3] {
    [0, 1, 2]
}

pub fn plane_indices() -> [usize; 6] {
    [0, 1, 2, 2, 1, 3]
}

pub fn cube_lines() -> [usize; 24] {
    [
        0, 1, 1, 3, 3, 2, 2, 0, 0, 4, 1, 5, 3, 7, 2, 6, 4, 5, 5, 7, 7, 6, 6, 4,
    ]
}

pub fn cube_indices() -> [usize; 36] {
    [
        0, 1, 2, 2, 1, 3, 1, 5, 3, 3, 5, 7, 2, 3, 6, 3, 7, 6, 4, 7, 5, 4, 6, 7, 0, 2, 4, 2, 6, 4,
        0, 4, 1, 1, 4, 5,
    ]
}

pub fn unit_cube_verts(size: f32) -> [Vec3; 24] {
    let size = size * 0.5;
    [
        vec3(-size, -size, size),
        vec3(size, -size, size),
        vec3(size, size, size),
        vec3(-size, size, size),
        vec3(-size, size, -size),
        vec3(size, size, -size),
        vec3(size, -size, -size),
        vec3(-size, -size, -size),
        vec3(size, -size, -size),
        vec3(size, size, -size),
        vec3(size, size, size),
        vec3(size, -size, size),
        vec3(-size, -size, size),
        vec3(-size, size, size),
        vec3(-size, size, -size),
        vec3(-size, -size, -size),
        vec3(size, size, -size),
        vec3(-size, size, -size),
        vec3(-size, size, size),
        vec3(size, size, size),
        vec3(size, -size, size),
        vec3(-size, -size, size),
        vec3(-size, -size, -size),
        vec3(size, -size, -size),
    ]
}

pub fn unit_cube_normals() -> [Vec2; 24] {
    let norm = Vec2::one().normalize();
    [
        vec2(0., 0.),
        vec2(1., 0.),
        norm,
        vec2(0., 1.),
        vec2(1., 0.),
        vec2(0., 0.),
        vec2(0., 1.),
        norm,
        vec2(0., 0.),
        vec2(1., 0.),
        norm,
        vec2(0., 1.),
        vec2(1., 0.),
        vec2(0., 0.),
        vec2(0., 1.),
        norm,
        vec2(1., 0.),
        vec2(0., 0.),
        vec2(0., 1.),
        norm,
        vec2(0., 0.),
        vec2(1., 0.),
        norm,
        vec2(0., 1.),
    ]
}

pub fn unit_cube_indices() -> [usize; 36] {
    [
        0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
    ]
}

pub fn colored_triangle() -> [f32; 18] {
    //Position, Color
    [
        -1., -1., 0., 1., 0., 0., 1., -1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 1.,
    ]
}

pub fn colored_triangle_indices() -> [usize; 3] {
    [0, 1, 2]
}

pub fn colored_cube(size: f32) -> Vec<f32> {
    let cube = cube(size);

    let mut out = Vec::with_capacity(cube.len() * 6);

    let colors = [
        Colors::RED,
        Colors::BLUE,
        Colors::GREEN,
        Colors::CYAN,
        Colors::MAGENTA,
        Colors::YELLOW,
    ];

    cube.iter().enumerate().for_each(|(idx, vertex)| {
        out.push(vertex.x());
        out.push(vertex.y());
        out.push(vertex.z());
        let color = colors[idx % colors.len()];
        out.push(color[0] as f32 / 255.);
        out.push(color[1] as f32 / 255.);
        out.push(color[2] as f32 / 255.);
    });

    out
}

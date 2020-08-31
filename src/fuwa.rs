use glam::*;
use pixels::{Error, Pixels, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;
use rayon::prelude::*;
use std::marker::{Send, Sync};

pub struct Fuwa<W: HasRawWindowHandle> {
    pub width: u32,
    pub height: u32,
    pub x_factor: f32,
    pub y_factor: f32,
    pixels: Pixels<W>,
}

struct FuwaPtr<W: HasRawWindowHandle>(*mut Fuwa<W>);

unsafe impl<W: HasRawWindowHandle> Send for FuwaPtr<W> {}
unsafe impl<W: HasRawWindowHandle> Sync for FuwaPtr<W> {}

impl<W: HasRawWindowHandle + Send + Sync> Fuwa<W> {
    fn get_self_ptr(&mut self) -> FuwaPtr<W> {
        FuwaPtr(self as *mut Self)
    }

    pub fn new(width: u32, height: u32, window: &W) -> Self {
        Self {
            width,
            height,
            x_factor: width as f32 * 0.5,
            y_factor: height as f32 * 0.5,
            pixels: Pixels::new(width, height, SurfaceTexture::new(width, height, &*window))
                .unwrap(),
        }
    }

    pub fn clear(&mut self, color: &[u8; 4]) {
        self.pixels
            .get_frame()
            .par_chunks_exact_mut(4)
            .for_each(|pixel| {
                pixel.copy_from_slice(color);
            })
    }

    pub fn render(&mut self) -> Result<(), Error> {
        self.pixels.render()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize(width, height);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        let index = self.pos_to_index(x, y);

        assert!(
            (index as u32) < self.width * self.height * 4,
            "Pixel drawn out of bounds!"
        );

        self.set_pixel_by_index(index, color)
    }

    fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: &[u8; 4]) {
        self.set_pixel_by_index(self.pos_to_index(x, y), color)
    }

    fn pos_to_index(&self, x: u32, y: u32) -> usize {
        4 * (x + (y * self.width)) as usize
    }

    fn set_pixel_by_index(&mut self, index: usize, color: &[u8; 4]) {
        unsafe {
            self.pixels
                .get_frame()
                .get_unchecked_mut(index..index + 4)
                .copy_from_slice(color);
        }
    }

    pub fn draw_triangle(&mut self, points: &[Vec3; 3], color: &[u8; 4]) {
        let mut min_x = points[0].x();
        let mut min_y = points[0].y();

        let mut max_x = min_x;
        let mut max_y = min_y;

        for point in points.iter().skip(1) {
            let test_x = point.x();
            let test_y = point.y();

            if test_x < min_x {
                min_x = test_x
            } else if test_x > max_x {
                max_x = test_x;
            }

            if test_y < min_y {
                min_y = test_y
            } else if test_y > max_y {
                max_y = test_y
            }
        }

        //clamp to view
        if min_x < 0. {
            min_x = 0.;
        }

        if min_y < 0. {
            min_y = 0.;
        }

        if max_x > self.width as f32 - 1. {
            max_x = self.width as f32 - 1.;
        }

        if max_y > self.height as f32 - 1. {
            max_y = self.height as f32 - 1.;
        }

        if is_degenerate(points, Vec2::new(points[0].x(), points[0].y())) {
            return;
        }

        unsafe {
            let self_ptr = self.get_self_ptr();
            (min_x.floor() as u32..=max_x.ceil() as u32)
                .into_par_iter()
                .for_each(|x| {
                    (min_y.floor() as u32..=max_y.ceil() as u32)
                        .into_par_iter()
                        .for_each(|y| {
                            let barycentric = barycentric(points, Vec2::new(x as f32, y as f32));
                            if !(barycentric.x() < 0.
                                || barycentric.y() < 0.
                                || barycentric.z() < 0.)
                            {
                                (*self_ptr.0).set_pixel_unchecked(x, y, color);
                            }
                        });
                });
        }
    }

    pub fn draw_line(&mut self, mut start: Vec3, mut end: Vec3, color: &[u8; 4]) {
        use std::ptr::swap;

        assert!(
            self.check_3d_pixel_within_bounds(&start) && self.check_3d_pixel_within_bounds(&end),
            "Line drawn out of bounds."
        );

        let start_ptr = &mut start as *mut Vec3;
        let end_ptr = &mut end as *mut Vec3;
        let steep = if f32::abs(start.x() - end.x()) < f32::abs(start.y() - end.y()) {
            unsafe {
                swap((*start_ptr).x_mut(), (*start_ptr).y_mut());
                swap((*end_ptr).x_mut(), (*end_ptr).y_mut());
            }
            true
        } else {
            false
        };

        if start.x() > end.x() {
            unsafe {
                swap(start_ptr, end_ptr);
            }
        }

        unsafe {
            let self_ptr = self.get_self_ptr();

            (start.x() as u32..=end.x() as u32)
                .into_par_iter()
                .for_each(|x| {
                    let slope = (x as f32 - start.x()) / (end.x() - start.x());
                    let y = (start.y() * (1. - slope) + end.y() * slope) as u32;
                    if steep {
                        (*self_ptr.0).set_pixel_unchecked(y, x, color);
                    } else {
                        (*self_ptr.0).set_pixel_unchecked(x, y, color);
                    }
                })
        }
    }

    pub fn transform_screen_space_perspective(&self, point: &mut Vec3) {
        let z_inverse = 1. / point.z();
        *point.x_mut() = (point.x() * z_inverse + 1.) * self.x_factor;
        *point.y_mut() = (-point.y() * z_inverse + 1.) * self.y_factor;
    }

    pub fn transform_screen_space_orthographic(&self, point: &mut Vec3) {
        *point.x_mut() = (point.x() + 1.) * self.x_factor;
        *point.y_mut() = (-point.y() + 1.) * self.y_factor;
    }

    fn check_pixel_within_bounds(&self, pos: &Vec2) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    fn check_3d_pixel_within_bounds(&self, pos: &Vec3) -> bool {
        let x = pos.x() as i32;
        let y = pos.y() as i32;

        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub fn draw_indexed(&mut self, verts: &[Vec3], indices: &[u32], color: &[u8; 4]) {
        unsafe {
            let self_ptr = self.get_self_ptr();
            indices.par_chunks_exact(3).for_each(|tri| {
                (*self_ptr.0).draw_triangle(
                    &[
                        *verts.get_unchecked(tri[0] as usize),
                        *verts.get_unchecked(tri[1] as usize),
                        *verts.get_unchecked(tri[2] as usize),
                    ],
                    color,
                );
            })
        }
    }
}

pub fn cube(size: f32) -> [Vec3; 8] {
    let side = size * 0.5;
    [
        Vec3::new(-side, -side, -side),
        Vec3::new(side, -side, -side),
        Vec3::new(-side, side, -side),
        Vec3::new(side, side, -side),
        Vec3::new(-side, -side, side),
        Vec3::new(side, -side, side),
        Vec3::new(-side, side, side),
        Vec3::new(side, side, side),
    ]
}

pub fn cube_lines() -> [u32; 24] {
    [
        0, 1, 1, 3, 3, 2, 2, 0, 0, 4, 1, 5, 3, 7, 2, 6, 4, 5, 5, 7, 7, 6, 6, 4,
    ]
}

pub fn cube_indices() -> [u32; 36] {
    [
        0, 2, 1, 2, 3, 1, 1, 3, 5, 3, 7, 5, 2, 6, 3, 3, 6, 7, 4, 5, 7, 4, 7, 6, 0, 4, 2, 2, 4, 6,
        0, 1, 4, 1, 5, 4,
    ]
}

fn barycentric(points: &[Vec3; 3], test: Vec2) -> Vec3 {
    let u = calc_barycentric(points, test);
    Vec3::new(1. - (u.x() + u.y()) / u.z(), u.y() / u.z(), u.x() / u.z())
}

fn is_degenerate(points: &[Vec3; 3], test: Vec2) -> bool {
    f32::abs(calc_barycentric(points, test).z()) < 1.
}

fn calc_barycentric(points: &[Vec3; 3], test: Vec2) -> Vec3 {
    let v1 = Vec3::new(
        points[2].x() - points[0].x(),
        points[1].x() - points[0].x(),
        points[0].x() - test.x(),
    );
    let v2 = Vec3::new(
        points[2].y() - points[0].y(),
        points[1].y() - points[0].y(),
        points[0].y() - test.y(),
    );

    v1.cross(v2)
}

#![deny(clippy::all)]

use fuwa::*;
use glam::*;
use pixels::Error;
use rayon::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const ROT_SPEED: f32 = 0.1;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut fuwa = Fuwa::new(WIDTH, HEIGHT, 4, true, None, &window);

    let lines = cube_lines();
    let indices = cube_indices();
    let cube_verts = cube(1.0);

    let tri = tri(1.0);
    let tri_indices = tri_indices();
    let plane_indices = plane_indices();

    let mut offset = Vec3A::new(0., 0., 2.);

    let mut rot_x = 0.0;
    let mut rot_y = 0.0;
    let mut rot_z = 0.0;

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            //x
            if input.key_held(VirtualKeyCode::W) {
                rot_x += ROT_SPEED;
            } else if input.key_held(VirtualKeyCode::S) {
                rot_x -= ROT_SPEED;
            }

            //y
            if input.key_held(VirtualKeyCode::A) {
                rot_y += ROT_SPEED;
            } else if input.key_held(VirtualKeyCode::D) {
                rot_y -= ROT_SPEED
            }

            //Z
            if input.key_held(VirtualKeyCode::Q) {
                rot_z += ROT_SPEED;
            } else if input.key_held(VirtualKeyCode::E) {
                rot_z -= ROT_SPEED
            }

            if input.key_held(VirtualKeyCode::R) {
                *offset.z_mut() += ROT_SPEED;
            } else if input.key_held(VirtualKeyCode::F) {
                *offset.z_mut() -= ROT_SPEED;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                fuwa.resize(size.width, size.height);
            }
        }

        match event {
            Event::RedrawRequested(_) => {
                // Draw the current frame
                fuwa.clear(&Colors::BLACK);

                let rotation = Mat3::from_rotation_x(rot_x)
                    * Mat3::from_rotation_y(rot_y)
                    * Mat3::from_rotation_z(rot_z);

                let mut active_model = cube_verts;

                active_model.par_iter_mut().for_each(|vertex| {
                    *vertex = rotation.mul_vec3a(*vertex);
                    *vertex += offset;
                    fuwa.transform_screen_space_perspective(vertex);
                });

                let color = &Colors::GREEN;
                fuwa.draw_indexed(&active_model, &indices, color);

                // unsafe {
                //     fuwa.draw_triangle(&[
                //         *active_model.get_unchecked(0),
                //         *active_model.get_unchecked(1),
                //         *active_model.get_unchecked(2),
                //     ], color);

                // }

                if fuwa
                    .render()
                    .map_err(|e| println!("render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::MainEventsCleared => {
                // Handle input events
                window.request_redraw();
            }
            _ => (),
        }
    })
}

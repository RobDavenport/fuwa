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
            .with_title("Hello FUWA")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut fuwa = Fuwa::new(WIDTH, HEIGHT, 4, true, None, &window);
    let vertex_descriptor = VertexDescriptor::new(
        vec![VertexDescriptorField::Vec3, VertexDescriptorField::Vec3],
        0,
    );

    let fragment_shader = FragmentShader {
        fragment_shader: |in_data| {
            [
                (in_data[3] * 255.) as u8,
                (in_data[4] * 255.) as u8,
                (in_data[5] * 255.) as u8,
                0xFF,
            ]
        },
    };

    let mut pipeline = Pipeline::new(vertex_descriptor, fragment_shader);

    //let tex_handle = fuwa.upload_texture(load_texture("box.png".to_string()));

    let lines = cube_lines();
    let cube_indices = cube_indices();
    let cube_verts = cube(1.0);
    let mut cube_data = vec3_into_float_slice(&cube_verts);

    let tri = tri(1.);
    let plane = plane(1.);
    let tri_indices = tri_indices();
    let plane_indices = plane_indices();

    let colored_triangle_data = colored_triangle();
    let colored_triangle_indices = colored_triangle_indices();

    let colored_cube = colored_cube(1.);

    let mut plane_data = vec3_into_float_slice(&plane);

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
                fuwa.clear_depth_buffer();

                let rotation = Mat3::from_rotation_x(rot_x)
                    * Mat3::from_rotation_y(rot_y)
                    * Mat3::from_rotation_z(rot_z);

                //let mut active_model = cube_verts;
                //let mut active_indices = cube_indices();

                let mut active_data = colored_cube.clone();

                let active_model = IndexedVertexList {
                    index_list: &cube_indices,
                    vertex_list: &mut active_data,
                };

                pipeline.bind_rotation(rotation);
                pipeline.bind_translation(offset);

                pipeline.draw(&mut fuwa, &active_model);

                // active_model.par_iter_mut().for_each(|vertex| {
                //     *vertex = rotation.mul_vec3a(*vertex);
                //     *vertex += offset;
                // });

                // let cull_flags = active_indices
                //     .par_chunks_exact(3)
                //     .map(|triangle| {
                //         is_backfacing_points(&[
                //             active_model[triangle[0] as usize],
                //             active_model[triangle[1] as usize],
                //             active_model[triangle[2] as usize],
                //         ])
                //     })
                //     .collect::<Vec<_>>();

                // active_model.par_iter_mut().for_each(|vertex| {
                //     fuwa.transform_screen_space_perspective(vertex);
                // });

                // let mut second_model = cube_verts;
                // second_model.par_iter_mut().for_each(|vertex| {
                //     *vertex = rotation.mul_vec3a(*vertex);
                //     *vertex += offset;
                //     *vertex.x_mut() = -vertex.x();
                //     fuwa.transform_screen_space_perspective(vertex);
                // });

                //let color = &Colors::GREEN;
                //fuwa.draw_indexed(&plane, &plane_indices, &Colors::WHITE);
                // fuwa.draw_indexed_parallel(
                //     &active_model,
                //     &active_indices,
                //     &cull_flags,
                //     &Colors::WHITE,
                // );
                //fuwa.draw_indexed_parallel(&second_model, &indices, &Colors::GREEN);

                // unsafe {
                //     fuwa.draw_triangle(&[
                //         *active_model.get_unchecked(0),
                //         *active_model.get_unchecked(1),
                //         *active_model.get_unchecked(2),
                //     ], color);

                // }

                if fuwa
                    .render()
                    //.render_depth_buffer()
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

pub fn load_texture(path: String) -> Texture {
    let image_bytes = std::fs::read(format!("./resources/{}", &path)).unwrap();
    let image_data = image::load_from_memory(&image_bytes).unwrap();
    let image_data = image_data.as_rgba8().unwrap();
    let dimensions = image_data.dimensions();

    Texture {
        data: image_data.to_vec(),
        width: dimensions.0,
        height: dimensions.1,
        format: TextureFormat::RGBA,
    }
}

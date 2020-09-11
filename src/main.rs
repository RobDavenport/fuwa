use fuwa::*;
use glam::*;
use pixels::Error;
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
        vec![VertexDescriptorField::Vec3, VertexDescriptorField::Vec2],
        0,
    );

    let set = 0;
    let binding = 0;

    let mut pipeline = Pipeline::new(vertex_descriptor, FragmentShader::textured(set, binding));
    fuwa.load_texture("box.png".to_string(), set, binding);

    let cube_data = unit_cube_uvs_into_data(1.);
    let cube_indices = unit_cube_indices();

    let mut offset = Vec3A::new(0., 0., 2.0);

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
                fuwa.clear();
                fuwa.clear_depth_buffer();

                let rotation = Mat3::from_rotation_x(rot_x)
                    * Mat3::from_rotation_y(rot_y)
                    * Mat3::from_rotation_z(rot_z);

                let mut active_data = cube_data.clone();

                let active_model = IndexedVertexList {
                    index_list: &cube_indices,
                    vertex_list: &mut active_data,
                };

                pipeline.bind_rotation(rotation);
                pipeline.bind_translation(offset);

                pipeline.draw(&mut fuwa, &active_model);

                if fuwa
                    .render()
                    //.render_depth_buffer()
                    .map_err(|e| println!("render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                };
            }
            Event::MainEventsCleared => {
                // Handle input events
                window.request_redraw();
            }
            _ => (),
        }
    })
}

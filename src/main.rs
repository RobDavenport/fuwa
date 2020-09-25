use crate::pipeline;
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

const ROT_SPEED: f32 = 0.03;

#[derive(Debug, Eq, PartialEq)]
enum Scene {
    TexturedCube,
    ColorBlendCube,
    Model,
    DepthTester,
    ComplexScene,
}

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

    let mut fuwa = Fuwa::new(WIDTH, HEIGHT, num_cpus::get(), true, None, &window);

    let mut scene = Scene::TexturedCube;

    let mut vertex_shader = BasicVertexShader::new();

    // let pipeline = Pipeline::new(
    //     vertex_descriptor,
    //     fragment_shader::textured(set, binding),
    //     Box::new(vertex_shader),
    // );

    let box_texture_handle = fuwa.load_texture("box.png".to_string());
    let doge_texture_handle = fuwa.load_texture("doge-bow.png".to_string());

    let mut plane_shader = Textured::new(box_texture_handle);
    let cube_shader = ColorBlend::new();

    let cube_data = colored_cube(1.);
    let cube_indices = cube_indices();

    let plane_data = textured_plane(2.);
    let plane_indices = plane_indices();

    let (model_data, model_indices, model_texture_handle) = fuwa.load_viking_room();
    let model_shader = Textured::new(model_texture_handle);

    //let cube_data = textured_plane(1.);
    //let cube_indices = tri_indices();

    let mut offset = Vec3A::new(0., 0., 35.0);

    let mut rot_x = 0.0;
    let mut rot_y = 0.0;
    let mut rot_z = 0.0;

    event_loop.run(move |event, _, control_flow| {
        //optick::start_capture();
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                //optick::stop_capture("perf");
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Key1) {
                change_scene(&mut scene, Scene::TexturedCube);
            } else if input.key_pressed(VirtualKeyCode::Key2) {
                change_scene(&mut scene, Scene::ColorBlendCube);
            } else if input.key_pressed(VirtualKeyCode::Key3) {
                change_scene(&mut scene, Scene::Model);
            } else if input.key_pressed(VirtualKeyCode::Key4) {
                change_scene(&mut scene, Scene::DepthTester);
            } else if input.key_pressed(VirtualKeyCode::Key5) {
                change_scene(&mut scene, Scene::ComplexScene);
            }

            if input.key_pressed(VirtualKeyCode::T) {
                if plane_shader.get_texture_handle() == box_texture_handle {
                    println!("Texture changed to Doge");
                    plane_shader.set_texture_handle(doge_texture_handle)
                } else {
                    println!("Texture changed to Box");
                    plane_shader.set_texture_handle(box_texture_handle)
                }
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
                fuwa.clear_all();

                let rotation = Mat3::from_rotation_x(rot_x)
                    * Mat3::from_rotation_y(rot_y)
                    * Mat3::from_rotation_z(rot_z);

                let active_cube = IndexedVertexList {
                    index_list: &cube_indices,
                    raw_vertex_list: &mut cube_data.clone(),
                };

                let active_plane = IndexedVertexList {
                    index_list: &plane_indices,
                    raw_vertex_list: &mut plane_data.clone(),
                };

                let active_model = IndexedVertexList {
                    index_list: &model_indices,
                    raw_vertex_list: &mut model_data.clone(),
                };

                vertex_shader.bind_translation(offset);
                vertex_shader.bind_rotation(rotation);

                pipeline::draw(&mut fuwa, &vertex_shader, 2, &active_model);
                fuwa.render(&model_shader, 2);

                //pipeline::draw(&mut fuwa, &vertex_shader, 0, &active_cube);
                //pipeline::draw(&mut fuwa, &vertex_shader, 1, &active_plane);

                //fuwa.render(&cube_shader, 0);
                //fuwa.render(&plane_shader, 1);

                if fuwa
                    .present()
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
    });
}

fn change_scene(scene: &mut Scene, next_scene: Scene) {
    if *scene != next_scene {
        println!("Changed scene to: {:?}", next_scene);
        *scene = next_scene;
    }
}

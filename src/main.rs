#![deny(clippy::all)]
#![forbid(unsafe_code)]

use fuwa::{Colors, Fuwa};
use pixels::{Error};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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

    let mut fuwa = Fuwa::new(WIDTH, HEIGHT, &window);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            fuwa.clear(&Colors::BLACK);

            fuwa.draw_line(13, 20, 80, 40, &Colors::WHITE);
            fuwa.draw_line(20, 13, 40, 80, &Colors::RED);
            fuwa.draw_line(80, 40, 13, 20, &Colors::RED);

            fuwa.draw_line(0, 0, WIDTH, HEIGHT, &Colors::GREEN);

            if fuwa
                .render()
                .map_err(|e| println!("render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                fuwa.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            window.request_redraw();
        }
    });
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fuwa::*;
use glam::*;
use rayon::prelude::*;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const SIZE: f32 = 1.0;

fn init_window() -> Fuwa<'static, Window> {
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("FUWA BENCHMARKS TEST")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&EventLoop::new())
            .unwrap()
    };

    Fuwa::new(WIDTH, HEIGHT, 4, false, None, &window)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut fuwa = init_window();

    let vertex_descriptor = VertexDescriptor::new(
        vec![VertexDescriptorField::Vec3, VertexDescriptorField::Vec3],
        0,
    );

    let mut pipeline = Pipeline::new(vertex_descriptor, FragmentShader::color_blend());

    let colored_cube = colored_cube(1.);
    let cube_indices = cube_indices();

    let translation = vec3a(0., 0., 2.0);

    let black = fuwa::colors::BLACK;
    let white = fuwa::colors::WHITE;

    let rotation = Mat3::from_cols(
        vec3(0.69670665, -0.40504977, -0.59205955),
        vec3(0.0, 0.8253356, -0.5646425),
        vec3(0.71735615, 0.39339018, 0.5750168),
    );

    let mut active_data = colored_cube.clone();
    pipeline.bind_translation(translation);
    pipeline.bind_rotation(rotation);

    let active_model = IndexedVertexList {
        index_list: &cube_indices,
        vertex_list: &mut active_data,
    };

    c.bench_function("full_render_loop", |b| {
        b.iter(|| {
            fuwa.clear();
            fuwa.clear_depth_buffer();
            fuwa.clear_fragments();

            pipeline.draw(black_box(&mut fuwa), black_box(&active_model));

            fuwa.render().unwrap();
        });
    });

    c.bench_function("render_scene", |b| {
        fuwa.clear();
        fuwa.clear_depth_buffer();
        fuwa.clear_fragments();

        pipeline.draw(black_box(&mut fuwa), black_box(&active_model));
        
        b.iter(|| {
            fuwa.render().unwrap();
        });
    });

    c.bench_function("rasterize_scene", |b| {
        fuwa.clear();
        fuwa.clear_depth_buffer();
        fuwa.clear_fragments();

        b.iter(|| {
            pipeline.draw(black_box(&mut fuwa), black_box(&active_model));
        });
    });

    c.bench_function("clear_screen", |b| {
        fuwa.clear();
        fuwa.clear_depth_buffer();
        fuwa.clear_fragments();

        b.iter(|| {
            fuwa.clear();
        });
    });

    c.bench_function("clear_depth_buffer", |b| {
        fuwa.clear();
        fuwa.clear_depth_buffer();
        fuwa.clear_fragments();

        b.iter(|| {
            fuwa.clear_depth_buffer();
        });
    });

    c.bench_function("clear_fragments", |b| {
        fuwa.clear();
        fuwa.clear_depth_buffer();
        fuwa.clear_fragments();

        b.iter(|| {
            fuwa.clear_fragments();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

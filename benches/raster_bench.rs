use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fuwa::*;
use glam::*;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const SIZE: f32 = 1.0;

fn criterion_benchmark(c: &mut Criterion) {
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("FUWA BENCHMARKS TEST")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&EventLoop::new())
            .unwrap()
    };

    let mut fuwa = Fuwa::new(WIDTH, HEIGHT, num_cpus::get(), false, None, &window);

    let frag_shader = ColorBlend::new();
    let mut vert_shader = BasicVertexShader::new();

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
    vert_shader.bind_translation(translation);
    vert_shader.bind_rotation(rotation);

    let active_model = IndexedVertexList {
        index_list: &cube_indices,
        raw_vertex_list: &mut active_data,
    };

    c.bench_function("clear_all", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();
            for _ in 0..iters {
                let start = Instant::now();
                fuwa.clear_all();
                let end = start.elapsed();

                pipeline::draw(
                    black_box(&mut fuwa),
                    black_box(&vert_shader),
                    black_box(0),
                    black_box(&active_model),
                );

                fuwa.render(black_box(&frag_shader), black_box(0));
                fuwa.present();

                total += end;
            }
            total
        });
    });

    c.bench_function("rasterize_scene", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();
            for _ in 0..iters {
                fuwa.clear_all();

                let start = Instant::now();
                pipeline::draw(
                    black_box(&mut fuwa),
                    black_box(&vert_shader),
                    black_box(0),
                    black_box(&active_model),
                );
                let end = start.elapsed();

                fuwa.render(black_box(&frag_shader), black_box(0));
                fuwa.present();

                total += end;
            }
            total
        });
    });

    c.bench_function("render_scene", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();
            for _ in 0..iters {
                fuwa.clear_all();

                let start = Instant::now();
                pipeline::draw(
                    black_box(&mut fuwa),
                    black_box(&vert_shader),
                    black_box(0),
                    black_box(&active_model),
                );
                let end = start.elapsed();

                fuwa.render(black_box(&frag_shader), black_box(0));
                fuwa.present();

                total += end;
            }
            total
        });
    });

    c.bench_function("present_scene", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();
            for _ in 0..iters {
                fuwa.clear_all();

                pipeline::draw(
                    black_box(&mut fuwa),
                    black_box(&vert_shader),
                    black_box(0),
                    black_box(&active_model),
                );

                fuwa.render(black_box(&frag_shader), black_box(0));

                let start = Instant::now();
                fuwa.present();
                let end = start.elapsed();

                total += end;
            }
            total
        });
    });

    c.bench_function("full_render_loop", |b| {
        b.iter(|| {
            fuwa.clear_all();

            pipeline::draw(
                black_box(&mut fuwa),
                black_box(&vert_shader),
                black_box(0),
                black_box(&active_model),
            );

            fuwa.render(black_box(&frag_shader), black_box(0));
            fuwa.present();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

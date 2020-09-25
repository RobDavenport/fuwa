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

    let mut vert_shader = BasicVertexShader::new();

    let (model_data, model_indices, model_texture_handle) = fuwa.load_viking_room();
    let model_shader = Textured::new(model_texture_handle);

    let translation = vec3a(0., 0., 35.0);

    let rotation = Mat3::from_cols(
        vec3(-0.91775537, -0.27071026, -0.2905874),
        vec3(0.0, 0.7316888, -0.68163884),
        vec3(0.39714617, -0.6255777, -0.6715113),
    );

    let mut active_data = model_data.clone();
    vert_shader.bind_translation(translation);
    vert_shader.bind_rotation(rotation);

    let active_model = IndexedVertexList {
        index_list: &model_indices,
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

                fuwa.render(black_box(&model_shader), black_box(0));
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

                fuwa.render(black_box(&model_shader), black_box(0));
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

                pipeline::draw(
                    black_box(&mut fuwa),
                    black_box(&vert_shader),
                    black_box(0),
                    black_box(&active_model),
                );

                let start = Instant::now();
                fuwa.render(black_box(&model_shader), black_box(0));
                let end = start.elapsed();

                fuwa.present();

                total += end;
            }
            total
        });
    });

    // c.bench_function("present_scene", |b| {
    //     b.iter_custom(|iters| {
    //         let mut total = Duration::default();
    //         for _ in 0..iters {
    //             fuwa.clear_all();

    //             pipeline::draw(
    //                 black_box(&mut fuwa),
    //                 black_box(&vert_shader),
    //                 black_box(0),
    //                 black_box(&active_model),
    //             );

    //             fuwa.render(black_box(&model_shader), black_box(0));

    //             let start = Instant::now();
    //             fuwa.present();
    //             let end = start.elapsed();

    //             total += end;
    //         }
    //         total
    //     });
    // });

    // c.bench_function("full_render_loop", |b| {
    //     b.iter(|| {
    //         fuwa.clear_all();

    //         pipeline::draw(
    //             black_box(&mut fuwa),
    //             black_box(&vert_shader),
    //             black_box(0),
    //             black_box(&active_model),
    //         );

    //         fuwa.render(black_box(&model_shader), black_box(0));
    //         fuwa.present();
    //     });
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

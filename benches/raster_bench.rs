use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fuwa::*;
use rayon::prelude::*;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const SIZE: f32 = 1.0;

fn init_window() -> Fuwa<Window> {
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("FUWA BENCHMARKS TEST")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&EventLoop::new())
            .unwrap()
    };

    Fuwa::new(WIDTH, HEIGHT, 4, false, Some(true), &window)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut fuwa = init_window();
    let black = fuwa::Colors::BLACK;
    let white = fuwa::Colors::WHITE;

    let cube_verts = fuwa::cube(SIZE);
    let tri_verts = fuwa::tri(SIZE);

    let cube_indices = fuwa::cube_indices();
    let plane_indices = fuwa::plane_indices();
    let tri_indices = fuwa::tri_indices();

    let offset = glam::vec3a(0., 0., 2.);

    let mut draw_tri = tri_verts.clone();
    draw_tri.iter_mut().for_each(|vertex| {
        *vertex += offset;
        fuwa.transform_screen_space_perspective(vertex);
    });

    let mut draw_cube = cube_verts.clone();
    draw_cube.iter_mut().for_each(|vertex| {
        *vertex += offset;
        fuwa.transform_screen_space_perspective(vertex);
    });

    c.bench_function("clear_screen", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
        });
    });

    c.bench_function("render", |b| {
        b.iter(|| {
            fuwa.render().unwrap();
        });
    });

    c.bench_function("clear_screen + render", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.render().unwrap();
        });
    });

    c.bench_function("draw_triangle", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.draw_triangle(black_box(&draw_tri), black_box(&white));
            fuwa.render().unwrap();
        })
    });

    c.bench_function("draw_triangle_fast", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.draw_triangle_fast(black_box(&draw_tri), black_box(&white));
            fuwa.render().unwrap();
        })
    });

    c.bench_function("draw_triangle_parallel", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.draw_triangle_parallel(black_box(&draw_tri), black_box(&white));
            fuwa.render().unwrap();
        })
    });

    c.bench_function("draw_indexed cube", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.draw_indexed(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });

    c.bench_function("draw_indexed cube parallel", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));
            fuwa.draw_indexed_parallel(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform cube", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform parallel cube", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.par_iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform + render cube", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.draw_indexed(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform + render cube (par)", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.draw_indexed_parallel(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform (par) + render cube", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.par_iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.draw_indexed(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });

    c.bench_function("transform (par) + render cube (par)", |b| {
        b.iter(|| {
            fuwa.clear(black_box(&black));

            let mut my_verts = cube_verts;
            my_verts.par_iter_mut().for_each(|vertex| {
                *vertex += offset;
                fuwa.transform_screen_space_perspective(vertex);
            });

            fuwa.draw_indexed_parallel(
                black_box(&draw_cube),
                black_box(&cube_indices),
                black_box(&white),
            );
            fuwa.render().unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

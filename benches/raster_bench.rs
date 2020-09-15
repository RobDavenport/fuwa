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

    c.bench_function("render_scene", |b| {
        let mut active_data = colored_cube.clone();
        pipeline.bind_translation(translation);
        pipeline.bind_rotation(rotation);

        let active_model = IndexedVertexList {
            index_list: &cube_indices,
            vertex_list: &mut active_data,
        };

        b.iter(|| {
            fuwa.clear();
            fuwa.clear_depth_buffer();
            fuwa.clear_fragments();

            pipeline.draw(black_box(&mut fuwa), black_box(&active_model));

            fuwa.render().unwrap();
        });
    });

    // let cube_verts = fuwa::cube(SIZE);
    // let tri_verts = fuwa::tri(SIZE);

    // let cube_indices = fuwa::cube_indices();
    // let plane_indices = fuwa::plane_indices();
    // let tri_indices = fuwa::tri_indices();

    // let offset = glam::vec3a(0., 0., 2.);

    // let mut draw_tri = tri_verts.clone();
    // draw_tri.iter_mut().for_each(|vertex| {
    //     *vertex += black_box(offset);
    //     fuwa.transform_screen_space_perspective(vertex);
    // });

    // let mut draw_cube = cube_verts.clone();
    // draw_cube.iter_mut().for_each(|vertex| {
    //     *vertex += black_box(offset);
    //     fuwa.transform_screen_space_perspective(vertex);
    // });

    // c.bench_function("clear_screen", |b| {
    //     b.iter(|| {
    //         fuwa.clear();
    //     });
    // });

    // c.bench_function("clear_screen_color", |b| {
    //     b.iter(|| {
    //         fuwa.clear_color(black_box(&black));
    //     });
    // });

    // c.bench_function("clear_depth_buffer", |b| {
    //     b.iter(|| {
    //         fuwa.clear_depth_buffer();
    //     })
    // });

    // c.bench_function("render", |b| {
    //     b.iter(|| {
    //         fuwa.render().unwrap();
    //     });
    // });

    // c.bench_function("clear_screen + render", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));
    //         fuwa.render().unwrap();
    //     });
    // });

    // c.bench_function("calculate raster bb", |b| {

    //     let points = &[
    //         cube_verts[cube_indices[0] as usize],
    //         cube_verts[cube_indices[1] as usize],
    //         cube_verts[cube_indices[2] as usize]];

    //     b.iter(|| {
    //         fuwa.calculate_raster_bb(black_box(points));
    //     });
    // });

    // c.bench_function("draw_triangle_fast", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));
    //         fuwa.draw_triangle_fast(black_box(&draw_tri), black_box(&white));
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("draw_triangle_parallel", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));
    //         fuwa.draw_triangle_parallel(black_box(&draw_tri), black_box(&white));
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("draw_indexed cube", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));
    //         fuwa.draw_indexed(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("draw_indexed cube parallel", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));
    //         fuwa.draw_indexed_parallel(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("transform cube", |b| {
    //     b.iter(|| {
    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });
    //     })
    // });

    // c.bench_function("transform parallel cube", |b| {
    //     b.iter(|| {
    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.par_iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });
    //     })
    // });

    // c.bench_function("transform + render cube", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));

    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });

    //         fuwa.draw_indexed(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("transform + render cube (par)", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));

    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });

    //         fuwa.draw_indexed_parallel(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("transform (par) + render cube", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));

    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.par_iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });

    //         fuwa.draw_indexed(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });

    // c.bench_function("transform (par) + render cube (par)", |b| {
    //     b.iter(|| {
    //         fuwa.clear(black_box(&black));

    //         let mut my_verts = black_box(cube_verts);
    //         my_verts.par_iter_mut().for_each(|vertex| {
    //             *vertex += black_box(offset);
    //             fuwa.transform_screen_space_perspective(vertex);
    //         });

    //         fuwa.draw_indexed_parallel(
    //             black_box(&draw_cube),
    //             black_box(&cube_indices),
    //             black_box(&white),
    //         );
    //         fuwa.render().unwrap();
    //     })
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

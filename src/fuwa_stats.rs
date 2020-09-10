use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct FuwaStats {
    total_vertices: AtomicU32,
    total_triangles: AtomicU32,
    culled_triangles: AtomicU32,
    draw_calls: AtomicU32,
}

impl FuwaStats {
    pub fn print_data(&self) {
        println!(
            "Total Vertices: {}",
            self.total_vertices.load(Ordering::Relaxed)
        );
        println!(
            "Total Triangles: {}",
            self.total_triangles.load(Ordering::Relaxed)
        );
        println!(
            "Culled Triangles: {}",
            self.culled_triangles.load(Ordering::Relaxed)
        );
        println!("Draw Calls: {}", self.draw_calls.load(Ordering::Relaxed));
    }

    pub fn reset(&mut self) {
        self.total_vertices.store(0, Ordering::Relaxed);
        self.total_triangles.store(0, Ordering::Relaxed);
        self.culled_triangles.store(0, Ordering::Relaxed);
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

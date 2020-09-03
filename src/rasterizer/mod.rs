mod edge;
pub(crate) use edge::*;

use glam::*;

use crate::Fuwa;

use raw_window_handle::HasRawWindowHandle;

impl<W: HasRawWindowHandle> Fuwa<W> {}

#[derive(Copy, Clone)]
pub(crate) struct RasterBoundingBox {
    pub(crate) min_x: u32,
    pub(crate) min_y: u32,
    pub(crate) max_x: u32,
    pub(crate) max_y: u32,
}

impl RasterBoundingBox {
  //Try using Glams built in max/min functions!
  //Avoid perf regressions!
    pub(crate) fn build(points: &[Vec3A; 3], screen_width: u32, screen_height: u32) -> Self {
        let mut min_x = points[0].x();
        let mut min_y = points[0].y();

        let mut max_x = min_x;
        let mut max_y = min_y;

        for point in points.iter().skip(1) {
            let test_x = point.x();
            let test_y = point.y();

            if test_x < min_x {
                min_x = test_x
            } else if test_x > max_x {
                max_x = test_x;
            }

            if test_y < min_y {
                min_y = test_y
            } else if test_y > max_y {
                max_y = test_y
            }
        }

        //clamp to view
        if min_x < 0. {
            min_x = 0.;
        }

        if min_y < 0. {
            min_y = 0.;
        }

        if max_x >= screen_width as f32 {
            max_x = screen_width as f32 - 1.;
        }

        if max_y >= screen_height as f32 {
            max_y = screen_height as f32 - 1.;
        }

        min_x = min_x.floor();
        min_y = min_y.floor();
        max_x = max_x.ceil();
        max_y = max_y.ceil();

        Self {
            min_x: min_x as u32,
            min_y: min_y as u32,
            max_x: max_x as u32,
            max_y: max_y as u32,
        }
    }
}

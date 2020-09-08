use glam::*;

#[derive(Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) one_step_x: Vec4,
    pub(crate) one_step_y: Vec4,
}

impl Edge {
    pub(crate) const STEP_X: usize = 4;
    pub(crate) const STEP_Y: usize = 1;

    pub fn init(v0: &Vec2, v1: &Vec2, origin: &Vec2) -> (Self, Vec4) {
        let a_val = v0.y() - v1.y();
        let b_val = v1.x() - v0.x();
        let c_val = (v0.x() * v1.y()) - (v0.y() * v1.x());

        let x = Vec4::splat(origin.x()) + vec4(0., 1., 2., 3.);
        let y = Vec4::splat(origin.y());

        let a_vec = Vec4::splat(a_val);
        let b_vec = Vec4::splat(b_val);
        let c_vec = Vec4::splat(c_val);

        (
            Self {
                one_step_x: a_vec * Self::STEP_X as f32,
                one_step_y: b_vec * Self::STEP_Y as f32,
            },
            a_vec * x + b_vec * y + c_vec,
        )
    }
}

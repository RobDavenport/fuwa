use glam::*;

#[derive(Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) one_step_x: Vec4,
    pub(crate) one_step_y: Vec4,
}

impl Edge {
    pub(crate) const STEP_X: usize = 4;
    pub(crate) const STEP_Y: usize = 1;

    pub fn init(v0: &Vec3A, v1: &Vec3A, origin: &Vec3A) -> (Self, Vec4) {
        let a = v0.y() - v1.y();
        let b = v1.x() - v0.x();
        let c = (v0.x() * v1.y()) - (v0.y() * v1.x());

        let x = Vec4::splat(origin.x()) + vec4(0., 1., 2., 3.);
        let y = Vec4::splat(origin.y());

        let a_vec = Vec4::splat(a);
        let b_vec = Vec4::splat(b);
        let c_vec = Vec4::splat(c);

        (
            Self {
                one_step_x: a_vec * Self::STEP_X as f32,
                one_step_y: b_vec * Self::STEP_Y as f32,
            },
            a_vec * x + b_vec * y + c_vec,
        )
    }
}

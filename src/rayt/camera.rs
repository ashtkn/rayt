use crate::rayt::*;

#[derive(Debug)]
pub struct Camera {
    pub origin: Point3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

impl Camera {
    pub fn new(u: Vec3, v: Vec3, w: Vec3) -> Self {
        Self {
            origin: Point3::zero(),
            u,
            v,
            w,
        }
    }

    pub fn from_look_at(
        origin: Vec3,
        look_at: Vec3,
        view_up: Vec3,
        view_fov: f64,
        aspect: f64,
    ) -> Self {
        let half_h = (view_fov.to_radians() * 0.5).tan();
        let half_w = aspect * half_h;
        let w = (origin - look_at).normalize();
        let u = view_up.cross(w).normalize();
        let v = w.cross(u);
        let uw = half_w * u;
        let vh = half_h * v;
        Self {
            origin,
            u: 2.0 * uw,
            v: 2.0 * vh,
            w: origin - uw - vh - w,
        }
    }

    pub fn ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.w + self.u * u + self.v * v - self.origin,
        }
    }
}

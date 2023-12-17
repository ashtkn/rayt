mod float3;
pub use self::float3::{Color, Float3, Point3, Vec3};

mod quat;
pub use self::quat::Quat;

mod ray;
pub use self::ray::Ray;

mod camera;
pub use self::camera::Camera;

pub use std::f64::consts::FRAC_1_PI;
pub use std::f64::consts::PI;

pub const PI2: f64 = PI * 2.0;
pub const EPS: f64 = 1e-6;

use rand::random;

use crate::rayt::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Float3([f64; 3]);

pub type Color = Float3;
pub type Vec3 = Float3;
pub type Point3 = Float3;

impl Float3 {
    pub fn x(&self) -> f64 {
        self.0[0]
    }
    pub fn y(&self) -> f64 {
        self.0[1]
    }
    pub fn z(&self) -> f64 {
        self.0[2]
    }
    pub const fn xaxis() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
    pub const fn yaxis() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }
    pub const fn zaxis() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

impl Float3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }
    pub const fn zero() -> Self {
        Self([0.0; 3])
    }
    pub const fn one() -> Self {
        Self([1.0; 3])
    }
    pub const fn fill(value: f64) -> Self {
        Self([value; 3])
    }
}

impl Float3 {
    pub fn sqrt(&self) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.sqrt()))
    }
    pub fn near_zero(&self) -> bool {
        self.0.iter().all(|x| x.abs() < EPS)
    }
    pub fn saturate(&self) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.min(1.0).max(0.0)))
    }
}

impl Float3 {
    pub fn to_array(self) -> [f64; 3] {
        self.0
    }
    pub fn iter(&self) -> std::slice::Iter<'_, f64> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, f64> {
        self.0.iter_mut()
    }
}

impl Float3 {
    pub fn dot(&self, rhs: Self) -> f64 {
        self.0
            .iter()
            .zip(rhs.0.iter())
            .fold(0.0, |acc, (l, r)| acc + l * r)
    }
    pub fn cross(&self, rhs: Self) -> Self {
        Self([
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() * rhs.y() - self.y() * rhs.x(),
        ])
    }
    pub fn length(&self) -> f64 {
        self.length_squered().sqrt()
    }
    pub fn length_squered(&self) -> f64 {
        self.0.iter().fold(0.0, |acc, x| acc + x * x)
    }
    pub fn normalize(&self) -> Self {
        *self / self.length()
    }
    pub fn lerp(&self, v: Self, t: f64) -> Self {
        *self + (v - *self) * t
    }
}

impl FromIterator<f64> for Float3 {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        let mut inner_itr = iter.into_iter();
        Self([
            inner_itr.next().unwrap(),
            inner_itr.next().unwrap(),
            inner_itr.next().unwrap(),
        ])
    }
}

impl std::ops::Neg for Float3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::from_iter(self.0.map(|x| -x))
    }
}

impl std::ops::AddAssign<Float3> for Float3 {
    fn add_assign(&mut self, rhs: Float3) {
        for i in 0..3 {
            self.0[i] += rhs.0[i];
        }
    }
}

impl std::ops::Add<Float3> for Float3 {
    type Output = Self;
    fn add(self, rhs: Float3) -> Self {
        Self::from_iter(self.iter().zip(rhs.iter()).map(|(l, r)| l + r))
    }
}

impl std::ops::SubAssign<Float3> for Float3 {
    fn sub_assign(&mut self, rhs: Float3) {
        for i in 0..3 {
            self.0[i] -= rhs.0[i];
        }
    }
}

impl std::ops::Sub<Float3> for Float3 {
    type Output = Self;
    fn sub(self, rhs: Float3) -> Self {
        Self::from_iter(self.iter().zip(rhs.iter()).map(|(l, r)| l - r))
    }
}

impl std::ops::Mul<Float3> for Float3 {
    type Output = Self;
    fn mul(self, rhs: Float3) -> Self {
        Self::from_iter(self.iter().zip(rhs.iter()).map(|(l, r)| l * r))
    }
}

impl std::ops::MulAssign<f64> for Float3 {
    fn mul_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            self.0[i] *= rhs;
        }
    }
}

impl std::ops::Mul<f64> for Float3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::from_iter(self.iter().map(|x| x * rhs))
    }
}

impl std::ops::Mul<Float3> for f64 {
    type Output = Float3;
    fn mul(self, rhs: Float3) -> Float3 {
        rhs * self
    }
}

impl std::ops::DivAssign<f64> for Float3 {
    fn div_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            self.0[i] /= rhs;
        }
    }
}

impl std::ops::Div<f64> for Float3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self::from_iter(self.iter().map(|x| x / rhs))
    }
}

impl Float3 {
    pub fn from_hex(hex: &[u8; 6]) -> Self {
        if let Ok(hex_str) = std::str::from_utf8(hex) {
            let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap();
            Self::from_rgb(r, g, b)
        } else {
            panic!();
        }
    }
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }

    pub fn to_rgb(self) -> [u8; 3] {
        [self.r(), self.g(), self.b()]
    }

    pub fn r(&self) -> u8 {
        (255.99 * self.0[0].min(1.0).max(0.0)) as u8
    }
    pub fn g(&self) -> u8 {
        (255.99 * self.0[1].min(1.0).max(0.0)) as u8
    }
    pub fn b(&self) -> u8 {
        (255.99 * self.0[2].min(1.0).max(0.0)) as u8
    }
}

impl Float3 {
    pub fn random() -> Self {
        Self::new(random::<f64>(), random::<f64>(), random::<f64>())
    }
    pub fn random_fill() -> Self {
        Self::fill(random::<f64>())
    }
    pub fn random_limit(min: f64, max: f64) -> Self {
        Self::from_iter(Self::random().0.iter().map(|x| min + x * (max - min)))
    }
    pub fn random_in_unit_sphere() -> Self {
        loop {
            let point = Self::random_limit(-1.0, 1.0);
            if point.length() < 1.0 {
                return point;
            }
        }
    }
}

impl Float3 {
    pub fn gamma(&self, factor: f64) -> Self {
        let recip = factor.recip();
        Self::from_iter(self.0.iter().map(|x| x.powf(recip)))
    }
    pub fn degamma(&self, factor: f64) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.powf(factor)))
    }
}

impl Float3 {
    pub fn reflect(&self, normal: Self) -> Self {
        *self - 2.0 * self.dot(normal) * normal
    }
}

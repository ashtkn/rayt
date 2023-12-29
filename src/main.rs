#![allow(dead_code)]

mod rayt;

use std::sync::Arc;

use crate::rayt::*;

struct HitInfo {
    t: f64,
    p: Point3,
    n: Vec3,
    m: Arc<dyn Material>,
    u: f64,
    v: f64,
}

impl HitInfo {
    const fn new(t: f64, p: Point3, n: Vec3, m: Arc<dyn Material>, u: f64, v: f64) -> Self {
        Self { t, p, n, m, u, v }
    }
}

trait Shape: Sync {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo>;
}

struct Sphere {
    center: Point3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    const fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Shape for Sphere {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * ray.direction.dot(oc);
        let c = oc.dot(oc) - self.radius.powi(2);
        let d = b * b - 4.0 * a * c;
        if d > 0.0 {
            // こちらの解のほうが始点に近いので先に判定
            let temp = (-b - d.sqrt()) / (2.0 * a);
            if t0 < temp && temp < t1 {
                let p = ray.at(temp);
                return Some(HitInfo::new(
                    temp,
                    p,
                    (p - self.center) / self.radius,
                    Arc::clone(&self.material),
                    0.0,
                    0.0,
                ));
            }
            // 始点から近いほうの解が光線の衝突範囲含まれないときは遠い方の解を評価
            let temp = (-b + d.sqrt()) / (2.0 * a);
            if t0 < temp && temp < t1 {
                let p = ray.at(temp);
                return Some(HitInfo::new(
                    temp,
                    p,
                    (p - self.center) / self.radius,
                    Arc::clone(&self.material),
                    0.0,
                    0.0,
                ));
            }
        }
        None
    }
}

struct ShapeList {
    pub objects: Vec<Box<dyn Shape>>,
}

impl ShapeList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
    pub fn push(&mut self, object: Box<dyn Shape>) {
        self.objects.push(object);
    }
}

impl Shape for ShapeList {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        let mut hit_info: Option<HitInfo> = None;
        let mut closest_so_far = t1;
        for object in &self.objects {
            if let Some(info) = object.hit(ray, t0, closest_so_far) {
                closest_so_far = info.t;
                hit_info = Some(info);
            }
        }
        hit_info
    }
}

trait Material: Sync + Send {
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo>;
}

struct ScatterInfo {
    ray: Ray,
    albedo: Color,
}

impl ScatterInfo {
    fn new(ray: Ray, albedo: Color) -> Self {
        Self { ray, albedo }
    }
}

struct Lambertian {
    albedo: Box<dyn Texture>,
}

impl Lambertian {
    fn new(albedo: Box<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let target = hit.p + hit.n + Vec3::random_in_unit_sphere();
        let albedo = self.albedo.value(hit.u, hit.v, hit.p);
        Some(ScatterInfo::new(Ray::new(hit.p, target - hit.p), albedo))
    }
}

struct Metal {
    albedo: Box<dyn Texture>,
    fuzz: f64,
}

impl Metal {
    fn new(albedo: Box<dyn Texture>, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let mut reflected = ray.direction.normalize().reflect(hit.n);
        reflected = reflected + self.fuzz * Vec3::random_in_unit_sphere();
        if reflected.dot(hit.n) > 0.0 {
            let albedo = self.albedo.value(hit.u, hit.v, hit.p);
            Some(ScatterInfo::new(Ray::new(hit.p, reflected), albedo))
        } else {
            None
        }
    }
}

struct Dielectric {
    ri: f64,
}

impl Dielectric {
    const fn new(ri: f64) -> Self {
        Self { ri }
    }
    // Schlick 近似
    fn schlick(cosine: f64, ri: f64) -> f64 {
        let r0 = ((1.0 - ri) / (1.0 + ri)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let reflected = ray.direction.reflect(hit.n);
        let (outward_normal, ni_over_nt, cosine) = {
            let dot = ray.direction.dot(hit.n);
            if dot > 0.0 {
                (-hit.n, self.ri, self.ri * dot / ray.direction.length())
            } else {
                (hit.n, self.ri.recip(), -dot / ray.direction.length())
            }
        };
        if let Some(refracted) = (-ray.direction).refract(outward_normal, ni_over_nt) {
            if Vec3::random_fill().x() > Self::schlick(cosine, self.ri) {
                return Some(ScatterInfo::new(Ray::new(hit.p, refracted), Color::one()));
            }
        }
        Some(ScatterInfo::new(Ray::new(hit.p, reflected), Color::one()))
    }
}

trait Texture: Sync + Send {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

struct ColorTexture {
    color: Color,
}

impl ColorTexture {
    const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for ColorTexture {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.color
    }
}

struct CheckerTexture {
    odd: Box<dyn Texture>,
    even: Box<dyn Texture>,
    freq: f64,
}

impl CheckerTexture {
    fn new(odd: Box<dyn Texture>, even: Box<dyn Texture>, freq: f64) -> Self {
        Self { odd, even, freq }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let sines = p.iter().fold(1.0, |acc, x| acc * (x * self.freq).sin());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

struct SimpleScene {
    world: ShapeList,
}

impl SimpleScene {
    fn new() -> Self {
        let mut world = ShapeList::new();
        world.push(
            ShapeBuilder::new()
                .color_texture(Color::new(0.1, 0.2, 0.5))
                .lambertian()
                .sphere(Point3::new(0.6, 0.0, -1.0), 0.5)
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(Color::new(0.8, 0.8, 0.8))
                .metal(0.4)
                .sphere(Point3::new(-0.6, 0.0, -1.0), 0.5)
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .checker_texture(Color::new(0.8, 0.8, 0.0), Color::new(0.8, 0.2, 0.0), 10.0)
                .lambertian()
                .sphere(Point3::new(0.0, -100.5, -1.0), 100.0)
                .build(),
        );
        Self { world }
    }
    fn background(&self, d: Vec3) -> Color {
        let t = 0.5 * (d.normalize().y() + 1.0);
        Color::one().lerp(Color::new(0.5, 0.7, 1.0), t)
    }
}

impl SceneWithDepth for SimpleScene {
    fn camera(&self) -> Camera {
        Camera::new(
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(-2.0, -1.0, -1.0),
        )
    }
    fn trace(&self, ray: Ray, depth: usize) -> Color {
        let hit_info = self.world.hit(&ray, 0.001, f64::MAX);
        if let Some(hit) = hit_info {
            let scatter_info = if depth > 0 {
                hit.m.scatter(&ray, &hit)
            } else {
                None
            };
            if let Some(scatter) = scatter_info {
                return scatter.albedo * self.trace(scatter.ray, depth - 1);
            } else {
                return Color::zero();
            }
        } else {
            self.background(ray.direction)
        }
    }
}

struct ShapeBuilder {
    texture: Option<Box<dyn Texture>>,
    material: Option<Arc<dyn Material>>,
    shape: Option<Box<dyn Shape>>,
}

impl ShapeBuilder {
    fn new() -> Self {
        Self {
            texture: None,
            material: None,
            shape: None,
        }
    }

    // textures

    fn color_texture(mut self, color: Color) -> Self {
        self.texture = Some(Box::new(ColorTexture::new(color)));
        self
    }

    fn checker_texture(mut self, odd_color: Color, even_color: Color, freq: f64) -> Self {
        self.texture = Some(Box::new(CheckerTexture::new(
            Box::new(ColorTexture::new(odd_color)),
            Box::new(ColorTexture::new(even_color)),
            freq,
        )));
        self
    }

    // Material

    fn lambertian(mut self) -> Self {
        self.material = Some(Arc::new(Lambertian::new(self.texture.unwrap())));
        self.texture = None;
        self
    }
    fn metal(mut self, fuzz: f64) -> Self {
        self.material = Some(Arc::new(Metal::new(self.texture.unwrap(), fuzz)));
        self.texture = None;
        self
    }
    fn dielectric(mut self, ri: f64) -> Self {
        self.material = Some(Arc::new(Dielectric::new(ri)));
        self
    }

    // shapes

    fn sphere(mut self, center: Point3, radius: f64) -> Self {
        self.shape = Some(Box::new(Sphere::new(
            center,
            radius,
            self.material.unwrap(),
        )));
        self.material = None;
        self
    }

    // build

    fn build(self) -> Box<dyn Shape> {
        self.shape.unwrap()
    }
}

// struct RandomScene {
//     world: ShapeList,
// }

// impl RandomScene {
//     fn new() -> Self {
//         let mut world = ShapeList::new();
//         world.push(
//             ShapeBuilder::new()
//                 .lambertian(Color::new(0.5, 0.5, 0.5))
//                 .sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0)
//                 .build(),
//         );
//         // Small spheres
//         for au in -11..11 {
//             let a = au as f64;
//             for bu in -11..11 {
//                 let b = bu as f64;
//                 let [rx, rz, material_choice] = Float3::random().to_array();
//                 let center = Point3::new(a + 0.9 * rx, 0.2, b + 0.9 * rz);
//                 if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
//                     world.push({
//                         if material_choice < 0.8 {
//                             let albedo = Color::random() * Color::random();
//                             ShapeBuilder::new()
//                                 .lambertian(albedo)
//                                 .sphere(center, 0.2)
//                                 .build()
//                         } else if material_choice < 0.95 {
//                             let albedo = Color::random_limit(0.5, 1.0);
//                             let fuzz = Float3::random_fill().x();
//                             ShapeBuilder::new()
//                                 .metal(albedo, fuzz)
//                                 .sphere(center, 0.2)
//                                 .build()
//                         } else {
//                             ShapeBuilder::new()
//                                 .dielectric(1.5)
//                                 .sphere(center, 0.2)
//                                 .build()
//                         }
//                     });
//                 }
//             }
//         }

//         // Big spheres
//         world.push(
//             ShapeBuilder::new()
//                 .dielectric(1.5)
//                 .sphere(Point3::new(0.0, 1.0, 0.0), 1.0)
//                 .build(),
//         );
//         world.push(
//             ShapeBuilder::new()
//                 .lambertian(Color::new(0.4, 0.2, 0.1))
//                 .sphere(Point3::new(-4.0, 1.0, 0.0), 1.0)
//                 .build(),
//         );
//         world.push(
//             ShapeBuilder::new()
//                 .metal(Color::new(0.7, 0.6, 0.5), 0.0)
//                 .sphere(Point3::new(4.0, 1.0, 0.0), 1.0)
//                 .build(),
//         );

//         Self { world }
//     }

//     fn background(&self, d: Vec3) -> Color {
//         let t = 0.5 * (d.normalize().y() + 1.0);
//         Color::one().lerp(Color::new(0.5, 0.7, 1.0), t)
//     }
// }

// impl SceneWithDepth for RandomScene {
//     fn camera(&self) -> Camera {
//         Camera::from_look_at(
//             Point3::new(13.0, 2.0, 3.0),
//             Point3::new(0.0, 0.0, 0.0),
//             Vec3::yaxis(),
//             20.0,
//             self.aspect(),
//         )
//     }
//     fn trace(&self, ray: Ray, depth: usize) -> Color {
//         let hit_info = self.world.hit(&ray, 0.001, f64::MAX);
//         if let Some(hit) = hit_info {
//             let scatter_info = if depth > 0 {
//                 hit.m.scatter(&ray, &hit)
//             } else {
//                 None
//             };
//             if let Some(scatter) = scatter_info {
//                 scatter.albedo * self.trace(scatter.ray, depth - 1)
//             } else {
//                 Color::zero()
//             }
//         } else {
//             self.background(ray.direction)
//         }
//     }
// }

fn main() {
    render_aa_with_depth(SimpleScene::new());
}

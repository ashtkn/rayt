use crate::rayt::*;

use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::{fs, path::Path};

const IMAGE_WIDTH: u32 = 200;
const IMAGE_HEIGHT: u32 = 100;
const OUTPUT_FILENAME: &str = "render.png";
const BACKUP_FILENAME: &str = "render_back.png";
const SAMPLES_PER_PIXEL: usize = 8;
const GAMMA_FACTOR: f64 = 2.2;
const MAX_RAY_BOUNCE_DEPTH: usize = 50;

fn backup() {
    let output_path = Path::new(OUTPUT_FILENAME);
    if output_path.exists() {
        println!("backup {:?} -> {:?}", OUTPUT_FILENAME, BACKUP_FILENAME);
        fs::rename(OUTPUT_FILENAME, BACKUP_FILENAME).unwrap();
    }
}

pub trait Scene {
    fn camera(&self) -> Camera;
    fn trace(&self, ray: Ray) -> Color;
    fn width(&self) -> u32 {
        IMAGE_WIDTH
    }
    fn height(&self) -> u32 {
        IMAGE_HEIGHT
    }
    fn spp(&self) -> usize {
        SAMPLES_PER_PIXEL
    }
    fn aspect(&self) -> f64 {
        self.width() as f64 / self.height() as f64
    }
}

pub trait SceneWithDepth {
    fn camera(&self) -> Camera;
    fn trace(&self, ray: Ray, depth: usize) -> Color;
    fn width(&self) -> u32 {
        IMAGE_WIDTH
    }
    fn height(&self) -> u32 {
        IMAGE_HEIGHT
    }
    fn spp(&self) -> usize {
        SAMPLES_PER_PIXEL
    }
    fn aspect(&self) -> f64 {
        self.width() as f64 / self.height() as f64
    }
}

pub fn render(scene: impl Scene + Sync) {
    backup();

    let camera = scene.camera();
    let w = scene.width();
    let h = scene.height();
    let mut img = RgbImage::new(w, h);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let u = *x as f64 / (w - 1) as f64;
            let v = (h - *y - 1) as f64 / (h - 1) as f64;
            let ray = camera.ray(u, v);
            let rgb = scene.trace(ray).to_rgb();
            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        });
    img.save(OUTPUT_FILENAME).unwrap();
    draw_in_window(BACKUP_FILENAME, img).unwrap();
}

pub fn render_aa(scene: impl Scene + Sync) {
    backup();

    let camera = scene.camera();
    let w = scene.width();
    let h = scene.height();
    let mut img = RgbImage::new(w, h);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut pixel_color = (0..scene.spp()).into_iter().fold(Color::zero(), |acc, _| {
                let [rx, ry, _] = Float3::random().to_array();
                let u = (*x as f64 + rx) / (w - 1) as f64;
                let v = ((h - *y - 1) as f64 + ry) / (h - 1) as f64;
                let ray = camera.ray(u, v);
                acc + scene.trace(ray)
            });
            pixel_color /= scene.spp() as f64;
            let rgb = pixel_color.gamma(GAMMA_FACTOR).to_rgb();
            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        });
    img.save(OUTPUT_FILENAME).unwrap();
    draw_in_window(BACKUP_FILENAME, img).unwrap();
}

pub fn render_aa_with_depth(scene: impl SceneWithDepth + Sync) {
    backup();

    let camera = scene.camera();
    let w = scene.width();
    let h = scene.height();
    let mut img = RgbImage::new(w, h);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut pixel_color = (0..scene.spp()).into_iter().fold(Color::zero(), |acc, _| {
                let [rx, ry, _] = Float3::random().to_array();
                let u = (*x as f64 + rx) / (w - 1) as f64;
                let v = ((h - *y - 1) as f64 + ry) / (h - 1) as f64;
                let ray = camera.ray(u, v);
                acc + scene.trace(ray, MAX_RAY_BOUNCE_DEPTH)
            });
            pixel_color /= scene.spp() as f64;
            let rgb = pixel_color.gamma(GAMMA_FACTOR).to_rgb();
            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        });
    img.save(OUTPUT_FILENAME).unwrap();
    draw_in_window(BACKUP_FILENAME, img).unwrap();
}

use wasm_bindgen::prelude::*;

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

#[derive(Clone, Copy)]
struct Vec3 { x: f64, y: f64, z: f64 }

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self { Vec3 { x, y, z } }
    fn dot(self, o: Vec3) -> f64 { self.x*o.x + self.y*o.y + self.z*o.z }
    fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x-o.x, self.y-o.y, self.z-o.z) }
    fn add(self, o: Vec3) -> Vec3 { Vec3::new(self.x+o.x, self.y+o.y, self.z+o.z) }
    fn scale(self, t: f64) -> Vec3 { Vec3::new(self.x*t, self.y*t, self.z*t) }
    fn length(self) -> f64 { self.dot(self).sqrt() }
    fn normalize(self) -> Vec3 { self.scale(1.0 / self.length()) }
    fn reflect(self, n: Vec3) -> Vec3 { self.sub(n.scale(2.0 * self.dot(n))) }
}

struct Ray { origin: Vec3, direction: Vec3 }

impl Ray {
    fn at(&self, t: f64) -> Vec3 { self.origin.add(self.direction.scale(t)) }
}

fn hit_sphere(center: Vec3, radius: f64, ray: &Ray) -> Option<f64> {
    let oc = ray.origin.sub(center);
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let disc = b*b - 4.0*a*c;
    if disc < 0.0 { return None; }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t > 0.001 { Some(t) } else { None }
}

fn shade(normal: Vec3, view_dir: Vec3, light_dir: Vec3) -> (f64, f64, f64) {
    let ambient = 0.08;
    let diff = normal.dot(light_dir).max(0.0);
    let reflect_dir = light_dir.scale(-1.0).reflect(normal);
    let spec = reflect_dir.dot(view_dir).max(0.0).powf(32.0);
    let r = (ambient + 0.7 * diff * 0.95 + 0.6 * spec).min(1.0);
    let g = (ambient + 0.7 * diff * 0.55 + 0.6 * spec).min(1.0);
    let b = (ambient + 0.7 * diff * 0.25 + 0.6 * spec).min(1.0);
    (r, g, b)
}

fn ray_color(ray: &Ray, light_dir: Vec3) -> (u8, u8, u8) {
    let center = Vec3::new(0.0, 0.0, -1.0);
    if let Some(t) = hit_sphere(center, 0.5, ray) {
        let hit = ray.at(t);
        let normal = hit.sub(center).normalize();
        let view_dir = ray.direction.scale(-1.0).normalize();
        let (r, g, b) = shade(normal, view_dir, light_dir);
        ((r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8)
    } else {
        let unit = ray.direction.normalize();
        let t = 0.5 * (unit.y + 1.0);
        let r = ((0.05 + 0.10 * t) * 255.0) as u8;
        let g = ((0.05 + 0.15 * t) * 255.0) as u8;
        let b = ((0.12 + 0.25 * t) * 255.0) as u8;
        (r, g, b)
    }
}

/// 渲染一帧，返回 RGBA 字节数组（WIDTH × HEIGHT × 4 字节）
/// time 驱动光源旋转角度
#[wasm_bindgen]
pub fn render_frame(time: f64) -> Vec<u8> {
    let light_dir = Vec3::new(
        time.cos(),
        0.6 + 0.4 * (time * 0.7).sin(),
        time.sin(),
    ).normalize();

    let origin = Vec3::new(0.0, 0.0, 0.0);
    let lower_left = Vec3::new(-1.0, -1.0, -1.0);
    let horizontal = Vec3::new(2.0, 0.0, 0.0);
    let vertical   = Vec3::new(0.0, 2.0, 0.0);

    let mut pixels = vec![0u8; WIDTH * HEIGHT * 4];

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let u = i as f64 / (WIDTH - 1) as f64;
            let v = 1.0 - j as f64 / (HEIGHT - 1) as f64;

            let direction = lower_left
                .add(horizontal.scale(u))
                .add(vertical.scale(v))
                .sub(origin);

            let ray = Ray { origin, direction };
            let (r, g, b) = ray_color(&ray, light_dir);

            let idx = (j * WIDTH + i) * 4;
            pixels[idx]     = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }

    pixels
}

#[wasm_bindgen]
pub fn canvas_size() -> u32 { WIDTH as u32 }

// # light — 光的本质（图形化版）
//
// 光源绕着球转动，你能看到：
//   - 高光点跟着光源走
//   - 阴影面永远背对光
//   - 边缘永远是渐变，不是硬边——因为法线在那里几乎垂直于光
//
// 这不是"画"出来的效果，是数学自然涌现的结果。

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

// --- 向量 ---

#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self { Vec3 { x, y, z } }
    fn dot(self, o: Vec3) -> f64 { self.x*o.x + self.y*o.y + self.z*o.z }
    fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x-o.x, self.y-o.y, self.z-o.z) }
    fn add(self, o: Vec3) -> Vec3 { Vec3::new(self.x+o.x, self.y+o.y, self.z+o.z) }
    fn scale(self, t: f64) -> Vec3 { Vec3::new(self.x*t, self.y*t, self.z*t) }
    fn length(self) -> f64 { self.dot(self).sqrt() }
    fn normalize(self) -> Vec3 { self.scale(1.0 / self.length()) }
    fn reflect(self, normal: Vec3) -> Vec3 {
        // 反射：r = d - 2(d·n)n
        self.sub(normal.scale(2.0 * self.dot(normal)))
    }
}

// --- 光线 ---

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f64) -> Vec3 {
        self.origin.add(self.direction.scale(t))
    }
}

// --- 第一步：光线和球相交吗？ ---
//
// 球面上所有点满足：|P - center|² = r²
// 光线上所有点满足：P = origin + t × direction
//
// 把光线代入球面方程，得到关于 t 的二次方程：
//   at² + bt + c = 0
//
// 判别式 disc = b²-4ac 就是答案：
//   disc < 0  → 光线完全错过球（没有实数解）
//   disc ≥ 0  → 光线击中球，t 就是"走了多远才碰到"
//
// 注意：这里没有任何"圆"的概念，只有代数。
// 屏幕上看到圆形轮廓，是因为边缘像素的射线恰好擦过球，
// 这是方程的解的几何形状，不是我们画的。

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

// --- 第二步：击中点有多亮？ ---
//
// 击中点的"法线"= 从球心指向击中点的方向向量。
// 它告诉我们：这个点的球面朝哪里。
//
// 漫反射亮度 = 法线 · 光方向（点积 = cos夹角）
//   法线正对光源 → cos=1 → 最亮
//   法线垂直光源 → cos=0 → 全暗（球的边缘就是这样）
//   法线背对光源 → cos<0 → 截断为0，背光面
//
// 这就是为什么球有立体感：不同位置的法线方向不同，
// 亮度自然就不同，没有任何地方"手动画了渐变"。

fn shade(normal: Vec3, view_dir: Vec3, light_dir: Vec3) -> (f64, f64, f64) {
    let ambient = 0.08;

    // 漫反射：法线和光方向的夹角
    let diff = normal.dot(light_dir).max(0.0);

    // 镜面反射（高光）
    let reflect_dir = light_dir.scale(-1.0).reflect(normal);
    let spec = reflect_dir.dot(view_dir).max(0.0).powf(32.0);

    // 球的颜色：暖橙色
    let r = ambient + 0.7 * diff * 0.95 + 0.6 * spec;
    let g = ambient + 0.7 * diff * 0.55 + 0.6 * spec;
    let b = ambient + 0.7 * diff * 0.25 + 0.6 * spec;

    (r.min(1.0), g.min(1.0), b.min(1.0))
}

fn ray_color(ray: &Ray, light_dir: Vec3) -> u32 {
    let sphere_center = Vec3::new(0.0, 0.0, -1.0);

    if let Some(t) = hit_sphere(sphere_center, 0.5, ray) {
        let hit = ray.at(t);
        // 第三步：法线 = 击中点 - 球心，归一化
        // 这是整个渲染的核心：法线完全由几何决定，
        // 不需要任何"艺术"判断，数学自动给出正确答案。
        let normal = hit.sub(sphere_center).normalize();
        let view_dir = ray.direction.scale(-1.0).normalize();

        let (r, g, b) = shade(normal, view_dir, light_dir);

        let ri = (r * 255.0) as u32;
        let gi = (g * 255.0) as u32;
        let bi = (b * 255.0) as u32;
        (ri << 16) | (gi << 8) | bi
    } else {
        // 背景：深蓝渐变
        let unit = ray.direction.normalize();
        let t = 0.5 * (unit.y + 1.0);
        let r = (0.05 + 0.10 * t) * 255.0;
        let g = (0.05 + 0.15 * t) * 255.0;
        let b = (0.12 + 0.25 * t) * 255.0;
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

fn render(buffer: &mut Vec<u32>, light_dir: Vec3) {
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let lower_left = Vec3::new(-1.0, -1.0, -1.0);
    let horizontal = Vec3::new(2.0, 0.0, 0.0);
    let vertical   = Vec3::new(0.0, 2.0, 0.0);

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let u = i as f64 / (WIDTH - 1) as f64;
            let v = 1.0 - j as f64 / (HEIGHT - 1) as f64;  // 翻转 y

            let direction = lower_left
                .add(horizontal.scale(u))
                .add(vertical.scale(v))
                .sub(origin);

            let ray = Ray { origin, direction };
            buffer[j * WIDTH + i] = ray_color(&ray, light_dir);
        }
    }
}

fn main() {
    let mut window = Window::new(
        "light — 光的本质  |  ESC 退出",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap();

    window.set_target_fps(60);

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    let mut time = 0.0f64;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // 光源绕 Y 轴旋转，同时有一点上下起伏
        let angle = time;
        let light_dir = Vec3::new(
            angle.cos() * 1.0,
            0.6 + 0.4 * (time * 0.7).sin(),
            angle.sin() * 1.0,
        ).normalize();

        render(&mut buffer, light_dir);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        time += 0.03;
    }
}

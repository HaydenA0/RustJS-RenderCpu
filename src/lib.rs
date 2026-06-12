use glam::Vec2;
use std::cell::UnsafeCell;
use wasm_bindgen::prelude::*;

struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
    const fn new(v: T) -> Self {
        SyncCell(UnsafeCell::new(v))
    }
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Pixel>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Image {
        Image {
            width,
            height,
            pixels: vec![
                Pixel {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                };
                width as usize * height as usize
            ],
        }
    }
    pub fn at(&self, x: u32, y: u32) -> Pixel {
        self.pixels[(y * self.width + x) as usize]
    }
    pub fn set(&mut self, x: u32, y: u32, pixel: Pixel) {
        self.pixels[(y * self.width + x) as usize] = pixel;
    }
}

fn fragment_shader10(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
    let mut output = Pixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    let vec_x = Vec2::new(x as f32, y as f32);
    let vec_r = Vec2::new(width as f32, height as f32);

    let vec_p = (2.0 * vec_x - vec_r) / height as f32;

    let d = (0.7 - vec_p.length_squared()).abs();

    let mut vec_v = 5.0 * vec_p * (1.0 - d);

    for i in 1..=8 {
        let i_f = i as f32;

        let old_v = vec_v;

        vec_v.x += (i_f * old_v.y + time).cos() / i_f + 0.7;
        vec_v.y += (i_f * old_v.x + time + i_f).cos() / i_f + 0.7;

        let delta_color_factor = 0.2 * (vec_v.x - vec_v.y).abs();

        let delta_color_red = vec_v.x.sin() + 1.0;
        let delta_color_green = vec_v.y.sin() + 1.0;

        let delta_color_blue = vec_v.y.sin() + 1.0;

        output.r += delta_color_red * delta_color_factor;
        output.g += delta_color_green * delta_color_factor;
        output.b += delta_color_blue * delta_color_factor;
    }

    let n_x = (vec_p.y - 4.0 * d).exp();
    let n_y = (-vec_p.y - 4.0 * d).exp();
    let n_z = (-2.0 * vec_p.y - 4.0 * d).exp();

    output.r = (n_x / output.r).tanh();
    output.g = (n_y / output.g).tanh();
    output.b = (n_z / output.b).tanh();

    output
}
fn snoise2D(v: Vec2) -> f32 {
    let c = Vec2::new(0.211324865405187, 0.366025403784439);
    let d = Vec2::new(-0.577350269189626, 0.024390243902439);

    let s = (v.x + v.y) * c.y;
    let i = Vec2::new((v.x + s).floor(), (v.y + s).floor());
    let x0 = Vec2::new(v.x - i.x + (i.x + i.y) * c.x, v.y - i.y + (i.x + i.y) * c.x);

    let i1 = if x0.x > x0.y {
        Vec2::new(1.0, 0.0)
    } else {
        Vec2::new(0.0, 1.0)
    };
    let x1 = Vec2::new(x0.x + c.x - i1.x, x0.y + c.x - i1.y);
    let x2 = Vec2::new(x0.x + d.x, x0.y + d.x);

    let i_mod_x = i.x - (i.x * (1.0 / 289.0)).floor() * 289.0;
    let i_mod_y = i.y - (i.y * (1.0 / 289.0)).floor() * 289.0;

    let permute = |x: f32| -> f32 {
        let val = ((x * 34.0) + 1.0) * x;
        val - (val * (1.0 / 289.0)).floor() * 289.0
    };

    let p0 = permute(permute(i_mod_y) + i_mod_x);
    let p1 = permute(permute(i_mod_y + i1.y) + i_mod_x + i1.x);
    let p2 = permute(permute(i_mod_y + 1.0) + i_mod_x + 1.0);

    let m0 = (0.5 - (x0.x * x0.x + x0.y * x0.y)).max(0.0);
    let m1 = (0.5 - (x1.x * x1.x + x1.y * x1.y)).max(0.0);
    let m2 = (0.5 - (x2.x * x2.x + x2.y * x2.y)).max(0.0);

    let m0_q = m0 * m0 * m0 * m0;
    let m1_q = m1 * m1 * m1 * m1;
    let m2_q = m2 * m2 * m2 * m2;

    let g0_x = 2.0 * (p0 * d.y).fract() - 1.0;
    let g0_y = g0_x.abs() - 0.5;
    let g0_x = g0_x - (g0_x + 0.5).floor();

    let g1_x = 2.0 * (p1 * d.y).fract() - 1.0;
    let g1_y = g1_x.abs() - 0.5;
    let g1_x = g1_x - (g1_x + 0.5).floor();

    let g2_x = 2.0 * (p2 * d.y).fract() - 1.0;
    let g2_y = g2_x.abs() - 0.5;
    let g2_x = g2_x - (g2_x + 0.5).floor();

    let r0 = 1.79284291400159 - 0.85373472095314 * (g0_x * g0_x + g0_y * g0_y);
    let r1 = 1.79284291400159 - 0.85373472095314 * (g1_x * g1_x + g1_y * g1_y);
    let r2 = 1.79284291400159 - 0.85373472095314 * (g2_x * g2_x + g2_y * g2_y);

    let g0 = Vec2::new(g0_x * r0, g0_y * r0);
    let g1 = Vec2::new(g1_x * r1, g1_y * r1);
    let g2 = Vec2::new(g2_x * r2, g2_y * r2);

    130.0
        * (m0_q * (g0.x * x0.x + g0.y * x0.y)
            + m1_q * (g1.x * x1.x + g1.y * x1.y)
            + m2_q * (g2.x * x2.x + g2.y * x2.y))
}

fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
    let vec_x = Vec2::new(x as f32, y as f32);
    let vec_r = Vec2::new(width as f32, height as f32);

    let base_p = (vec_x - vec_r * 0.5) / height as f32;

    let vec_p = Vec2::new(
        base_p.x * 8.0 + base_p.y * -6.0,
        base_p.x * 6.0 + base_p.y * 8.0,
    );

    let noise_input = Vec2::new(vec_p.x + time * 7.0, vec_p.y);
    let f = 3.0 + snoise2D(noise_input);

    let mut o_r = 0.0;
    let mut o_g = 0.0;
    let mut o_b = 0.0;

    let mut vec_v = Vec2::new(0.0, 0.0);

    for i in 1..=20 {
        let i_f = i as f32;

        let angle_x = i_f * i_f + (time + vec_p.x * 0.1) * 0.03 + i_f * 11.0;
        let angle_y = i_f * i_f + (time + vec_p.x * 0.1) * 0.03 + i_f * 9.0;
        vec_v = Vec2::new(vec_p.x + angle_x.cos() * 5.0, vec_p.y + angle_y.cos() * 5.0);

        let max_v = Vec2::new(vec_v.x.max(vec_v.x * f * 0.02), vec_v.y);
        let mut max_v_len = (max_v.x * max_v.x + max_v.y * max_v.y).sqrt();
        if max_v_len < 1e-6 {
            max_v_len = 1e-6;
        }

        let factor = (i_f * i_f + time).sin().exp() / max_v_len;

        o_r += ((i_f.sin() * 1.0).cos() + 1.0) * factor;
        o_g += ((i_f.sin() * 2.0).cos() + 1.0) * factor;
        o_b += ((i_f.sin() * 3.0).cos() + 1.0) * factor;
    }

    Pixel {
        r: (o_r / 100.0).max(0.0).powf(1.5).tanh(),
        g: (o_g / 100.0).max(0.0).powf(1.5).tanh(),
        b: (o_b / 100.0).max(0.0).powf(1.5).tanh(),
    }
}

fn fragment_shader20(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
    let mut output = Pixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    let vec_x = Vec2::new(x as f32, y as f32);
    let vec_r = Vec2::new(width as f32, height as f32);
    let mut vec_p = (2.0 * vec_x - vec_r) / height as f32;
    vec_p.y = -vec_p.y;

    let val = 0.1 / (vec_p.length() - 0.5 + 0.01 / (vec_p.x - vec_p.y)).abs();

    output.r += val;
    output.g += val;
    output.b += val;

    output
}

static IMAGE: SyncCell<Option<Image>> = SyncCell::new(None);
static RGBA: SyncCell<Option<Vec<u8>>> = SyncCell::new(None);

#[wasm_bindgen]
pub fn render_frame(width: u32, height: u32, time: f64) -> Vec<u8> {
    let image = unsafe {
        let p = IMAGE.get();
        let needs_new = match &*p {
            None => true,
            Some(img) => img.width != width || img.height != height,
        };
        if needs_new {
            *p = Some(Image::new(width, height));
        }
        match &mut *p {
            Some(img) => img,
            None => unreachable!(),
        }
    };

    for x in 0..width {
        for y in 0..height {
            image.set(x, y, fragment_shader(x, y, width, height, time as f32));
        }
    }

    let rgba = unsafe {
        let p = RGBA.get();
        if (*p).is_none() {
            *p = Some(Vec::with_capacity((width * height * 4) as usize));
        }
        match &mut *p {
            Some(v) => v,
            None => unreachable!(),
        }
    };
    rgba.clear();

    for pixel in &image.pixels {
        rgba.push((pixel.r * 255.0) as u8);
        rgba.push((pixel.g * 255.0) as u8);
        rgba.push((pixel.b * 255.0) as u8);
        rgba.push(255);
    }

    rgba.clone()
}

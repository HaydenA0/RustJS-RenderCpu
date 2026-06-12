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

fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
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

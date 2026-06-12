use std::cell::UnsafeCell;
use wasm_bindgen::prelude::*;

mod frag1;
mod frag2;
mod frag3;
mod frag4;
mod frag5;

fn get_shader(index: u32) -> ShaderFn {
    match index {
        1 => frag1::fragment_shader,
        2 => frag2::fragment_shader,
        3 => frag3::fragment_shader,
        4 => frag4::fragment_shader,
        5 => frag5::fragment_shader,
        _ => frag1::fragment_shader,
    }
}

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

type ShaderFn = fn(u32, u32, u32, u32, f32) -> Pixel;

static IMAGE: SyncCell<Option<Image>> = SyncCell::new(None);
static RGBA: SyncCell<Option<Vec<u8>>> = SyncCell::new(None);

#[wasm_bindgen]
pub fn render_frame(width: u32, height: u32, time: f64, shader: u32) -> Vec<u8> {
    let shader_fn = get_shader(shader);

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
            image.set(x, y, shader_fn(x, y, width, height, time as f32));
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

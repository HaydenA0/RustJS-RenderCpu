use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn render_frame(width: u32, height: u32, time: f64) -> Vec<u8> {
    let mut image = Image::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let r = x as f32 / (width + 1) as f32;
            let g = y as f32 / (height + 1) as f32;
            let b = 0.0;
            image.set(x, y, Pixel { r, g, b });
        }
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for pixel in &image.pixels {
        rgba.push((pixel.r * 255.0) as u8);
        rgba.push((pixel.g * 255.0) as u8);
        rgba.push((pixel.b * 255.0) as u8);
        rgba.push(255);
    }
    rgba
}

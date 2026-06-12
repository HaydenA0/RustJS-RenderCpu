use crate::Pixel;
use glam::Vec2;

pub fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
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

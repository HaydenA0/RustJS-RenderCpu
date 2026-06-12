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

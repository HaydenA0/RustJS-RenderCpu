use crate::Pixel;
use glam::Vec2;

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

pub fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
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

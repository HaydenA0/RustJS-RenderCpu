use crate::Pixel;
use glam::{Vec3, Vec4};

pub fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
    let mut o = Vec4::ZERO;
    let mut i = 0.0;
    let mut z = 0.0;
    let mut z_right = 0.0;
    let mut d = 0.0;
    let mut f = 0.0;

    let r = Vec3::new(width as f32, height as f32, 0.5);
    let r_xyy = Vec3::new(r.x, r.y, r.y);

    let gl_frag_coord_x = x as f32 + 0.5;
    let gl_frag_coord_y = (height as f32) - (y as f32) - 0.5;

    let fc = Vec3::new(gl_frag_coord_x, gl_frag_coord_y, 0.5);
    let fc_right = Vec3::new(gl_frag_coord_x + 1.0, gl_frag_coord_y, 0.5);

    while i < 100.0 {
        i += 1.0;

        let mut p = z * (fc * 2.0 - r_xyy) / r.y;
        let mut c = p;
        p.z += 8.0;
        c.z *= 3.0;

        f = 1.0;
        while f < 9.0 {
            f += 1.0;
            let c_yzx = Vec3::new(c.y, c.z, c.x);
            let arg = c_yzx * f + Vec3::splat(z + time * 0.5);
            c += Vec3::new(arg.x.sin(), arg.y.sin(), arg.z.sin()) / f;
        }

        f = 0.1 + (0.2 * c.y + (p.y + 0.8).abs()).abs();
        d = (p.length() - 3.0).max(0.9 - (p - Vec3::new(-1.0, 1.0, 3.0)).length());
        z += f.min(d) / 7.0;

        let mut p_right = z_right * (fc_right * 2.0 - r_xyy) / r.y;
        let mut c_right = p_right;
        p_right.z += 8.0;
        c_right.z *= 3.0;

        let mut f_right = 1.0;
        while f_right < 9.0 {
            f_right += 1.0;
            let c_yzx_right = Vec3::new(c_right.y, c_right.z, c_right.x);
            let arg_right = c_yzx_right * f_right + Vec3::splat(z_right + time * 0.5);
            c_right += Vec3::new(arg_right.x.sin(), arg_right.y.sin(), arg_right.z.sin()) / f_right;
        }

        let f_step_right = 0.1 + (0.2 * c_right.y + (p_right.y + 0.8).abs()).abs();
        let d_step_right =
            (p_right.length() - 3.0).max(0.9 - (p_right - Vec3::new(-1.0, 1.0, 3.0)).length());
        z_right += f_step_right.min(d_step_right) / 7.0;

        let dfdx_z = z_right - z;
        let outline_term = (dfdx_z * r.y + z).min(0.0);
        let exp_term = (d * d / 0.1).exp();

        let term1 = Vec4::new(4.0, 6.0, 8.0 + z, 0.0) / f;
        o += term1 - Vec4::splat(outline_term / exp_term);
    }

    let o_mapped = o / 2000.0;
    Pixel {
        r: o_mapped.x.tanh(),
        g: o_mapped.y.tanh(),
        b: o_mapped.z.tanh(),
    }
}

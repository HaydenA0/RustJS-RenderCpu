use crate::Pixel;
use glam::{Vec2, Vec3};

pub fn fragment_shader(x: u32, y: u32, width: u32, height: u32, _time: f32) -> Pixel {
    // Helper closure to evaluate the base shader value at a given horizontal coordinate
    let calc_val = |px: u32| {
        let vec_x = Vec2::new(px as f32, y as f32);
        let vec_r = Vec2::new(width as f32, height as f32);
        let mut vec_p = (2.0 * vec_x - vec_r) / height as f32;
        vec_p.y = -vec_p.y;

        0.1 / (vec_p.length() - 0.5 + 0.01 / (vec_p.x - vec_p.y)).abs()
    };

    // Evaluate current pixel
    let val_curr = calc_val(x);

    // Calculate dFdx (horizontal screenspace derivative)
    // Falls back to backward difference if on the right edge of the screen
    let dfdx_val = if x + 1 < width {
        calc_val(x + 1) - val_curr
    } else if x > 0 {
        val_curr - calc_val(x - 1)
    } else {
        0.0
    };

    let mut col = Vec3::splat(val_curr);
    let dfdx_col = Vec3::splat(dfdx_val);

    // Post processing: col.rgb += dFdx(col.rgb) * vec3(3, 0, -3);
    col += dfdx_col * Vec3::new(3.0, 0.0, -3.0);

    Pixel {
        r: col.x,
        g: col.y,
        b: col.z,
    }
}

use crate::Pixel;
use glam::{Mat3, Vec2, Vec3};

const PATH_LENGTH: i32 = 4;
const SAMPLES_PER_PIXEL: i32 = 16;

const MAX_DIST: f32 = 1e10;
const EPSILON: f32 = 0.001;

const LAMBERTIAN: f32 = 0.0;

const GRID_SCALE: f32 = 7.0;
const GRID_FILL: f32 = 0.6;
const LIGHT_FILL: f32 = 0.9;

const MAT_TYPE: f32 = 1.0;
const MAT_ROUGHNESS: f32 = 0.5;

const COLOR_SCALE: f32 = 0.25;
const COLOR_OFFSET: f32 = 4.0;

const FLOOR_BRIGHTNESS: f32 = 0.03;
const FLOOR_ROUGHNESS: f32 = 0.2;
const FLOOR_TYPE: f32 = 2.0;

#[inline]
fn fract(p: f32) -> f32 {
    p - p.floor()
}

#[inline]
fn fract_v2(v: Vec2) -> Vec2 {
    v - v.floor()
}

#[inline]
fn fract_v3(v: Vec3) -> Vec3 {
    v - v.floor()
}

#[inline]
fn hash1(p: f32) -> f32 {
    let mut p = fract(p * 0.1031);
    p *= p + 33.33;
    p *= p + p;
    fract(p)
}

#[inline]
fn hash2(seed: &mut f32) -> Vec2 {
    *seed += 1.0;
    let mut p3 = fract_v3(Vec3::splat(*seed) * Vec3::new(0.1031, 0.1030, 0.0973));
    let d = p3.dot(Vec3::new(p3.y, p3.z, p3.x) + Vec3::splat(33.33));
    p3 += Vec3::splat(d);

    let a = Vec2::new(p3.x, p3.x) + Vec2::new(p3.y, p3.z);
    let b = Vec2::new(p3.z, p3.y);
    fract_v2(a * b)
}

#[inline]
fn sign_v3(v: Vec3) -> Vec3 {
    Vec3::new(
        if v.x > 0.0 {
            1.0
        } else if v.x < 0.0 {
            -1.0
        } else {
            0.0
        },
        if v.y > 0.0 {
            1.0
        } else if v.y < 0.0 {
            -1.0
        } else {
            0.0
        },
        if v.z > 0.0 {
            1.0
        } else if v.z < 0.0 {
            -1.0
        } else {
            0.0
        },
    )
}

#[inline]
fn step_v3(edge: Vec3, x: Vec3) -> Vec3 {
    Vec3::new(
        if x.x < edge.x { 0.0 } else { 1.0 },
        if x.y < edge.y { 0.0 } else { 1.0 },
        if x.z < edge.z { 0.0 } else { 1.0 },
    )
}

#[inline]
fn rotate_2d(v: Vec2, a: f32) -> Vec2 {
    let cos = a.cos();
    let sin = a.sin();
    Vec2::new(v.x * cos + v.y * sin, -v.x * sin + v.y * cos)
}

#[inline]
fn glsl_mod(a: f32, b: f32) -> f32 {
    a - b * (a / b).floor()
}

#[inline]
fn checker_board(p: Vec2) -> f32 {
    glsl_mod(p.x.floor() + p.y.floor(), 2.0)
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

fn i_plane(
    ro: Vec3,
    rd: Vec3,
    dist_bound: Vec2,
    normal: &mut Vec3,
    plane_normal: Vec3,
    plane_dist: f32,
) -> f32 {
    let a = rd.dot(plane_normal);
    let d = -(ro.dot(plane_normal) + plane_dist) / a;
    if a > 0.0 || d < dist_bound.x || d > dist_bound.y {
        MAX_DIST
    } else {
        *normal = plane_normal;
        d
    }
}

fn grid(
    ro: Vec3,
    rd: Vec3,
    dist_bound: Vec2,
    normal: &mut Vec3,
    time: f32,
    mat_hash: &mut f32,
) -> f32 {
    let cube_grid_scale = GRID_SCALE;
    const STEPS: i32 = 20;

    let ros = ro * cube_grid_scale;
    let inside_unit_cube = ro.x.abs() < 1.0 && ro.y.abs() < 1.0 && ro.z.abs() < 1.0;
    let offset = if inside_unit_cube {
        0.0
    } else {
        dist_bound.x - EPSILON
    };

    let mut pos = (ros + offset * cube_grid_scale * rd).floor();
    let ri = Vec3::ONE / rd;
    let rs = sign_v3(rd);
    let mut dis = (pos - ros + Vec3::splat(0.5) + rs * 0.5) * ri;

    let mut hit = false;
    let mut mm = Vec3::ZERO;
    let mut grid_seed = 0.0;

    for _ in 0..STEPS {
        let dis_yxy = Vec3::new(dis.y, dis.x, dis.y);
        let dis_zzx = Vec3::new(dis.z, dis.z, dis.x);
        mm = step_v3(dis, dis_yxy) * step_v3(dis, dis_zzx);
        dis += mm * rs * ri;
        pos += mm * rs;

        let hash_pos = (pos + Vec3::splat(0.5)).abs();

        grid_seed = hash_pos.dot(Vec3::new(2.0, 2.0, 3.0));
        grid_seed += (time * 0.1 + grid_seed * (1.0 / GRID_SCALE)).floor();
        let grid_hash = hash1(grid_seed);

        let pos_p5_abs = (pos + Vec3::splat(0.5)).abs();
        let inside_cube = pos_p5_abs.x < cube_grid_scale
            && pos_p5_abs.y < cube_grid_scale
            && pos_p5_abs.z < cube_grid_scale;

        if inside_cube && grid_hash > GRID_FILL {
            hit = true;
            break;
        } else {
            let max_coord = pos.x.abs().max(pos.y.abs().max(pos.z.abs()));
            if max_coord > cube_grid_scale + 1.0 {
                break;
            }
        }
    }

    if hit {
        let mini = (pos - ros + Vec3::splat(0.5) - 0.5 * rs) * ri;
        let t = mini.x.max(mini.y.max(mini.z)) / cube_grid_scale;

        *mat_hash = hash1(-grid_seed);
        *normal = -mm * rs;
        t
    } else {
        MAX_DIST
    }
}

fn i_box(
    mut ro: Vec3,
    mut rd: Vec3,
    dist_bound: Vec2,
    normal: &mut Vec3,
    box_size: Vec3,
    time: f32,
    mat_hash: &mut f32,
) -> f32 {
    let c0 = Vec3::new(0.8164965, -0.5773504, -0.0000000);
    let c1 = Vec3::new(0.4082484, 0.5773502, -0.7071068);
    let c2 = Vec3::new(0.4082484, 0.5773502, 0.7071068);
    let rf = Mat3::from_cols(c0, c1, c2);

    let ro_xz = rotate_2d(Vec2::new(ro.x, ro.z), 0.1 * time);
    ro.x = ro_xz.x;
    ro.z = ro_xz.y;
    ro = Vec3::new(ro.dot(c0), ro.dot(c1), ro.dot(c2));

    let rd_xz = rotate_2d(Vec2::new(rd.x, rd.z), 0.1 * time);
    rd.x = rd_xz.x;
    rd.z = rd_xz.y;
    rd = Vec3::new(rd.dot(c0), rd.dot(c1), rd.dot(c2));

    let abs_rd = rd.abs();
    let max_abs_rd = Vec3::new(abs_rd.x.max(1e-8), abs_rd.y.max(1e-8), abs_rd.z.max(1e-8));
    let m = sign_v3(rd) / max_abs_rd;
    let n = m * ro;
    let k = m.abs() * box_size;

    let t1 = -n - k;
    let t2 = -n + k;

    let t_n = t1.x.max(t1.y.max(t1.z));
    let t_f = t2.x.min(t2.y.min(t2.z));

    if t_n > t_f || t_f <= 0.0 {
        MAX_DIST
    } else {
        if t_n <= dist_bound.y {
            let t_bound = Vec2::new(if t_n < 0.0 { 0.0 } else { t_n }, t_f);
            let mut grid_normal = Vec3::ZERO;
            let final_t_n = grid(ro, rd, t_bound, &mut grid_normal, time, mat_hash);

            *normal = rf.mul_vec3(grid_normal);
            let normal_xz = rotate_2d(Vec2::new(normal.x, normal.z), -0.1 * time);
            normal.x = normal_xz.x;
            normal.z = normal_xz.y;

            if final_t_n >= dist_bound.x {
                final_t_n
            } else {
                MAX_DIST
            }
        } else {
            MAX_DIST
        }
    }
}

fn world_hit(
    ro: Vec3,
    rd: Vec3,
    dist: Vec2,
    normal: &mut Vec3,
    time: f32,
    mat_hash: &mut f32,
) -> Vec2 {
    let mut d = dist;

    let mut temp_normal_box = *normal;
    let d_box = i_box(
        ro - Vec3::new(0.0, 0.4, 0.0),
        rd,
        d,
        &mut temp_normal_box,
        Vec3::ONE,
        time,
        mat_hash,
    );
    if d_box < d.y {
        d.y = d_box;
        *normal = temp_normal_box;
    }

    let mut temp_normal_plane = *normal;
    let d_plane = i_plane(
        ro,
        rd,
        d,
        &mut temp_normal_plane,
        Vec3::new(0.0, 1.0, 0.0),
        1.35,
    );
    if d_plane < d.y {
        d.y = d_plane;
        *normal = temp_normal_plane;
    }

    d
}

fn cos_weighted_random_hemisphere_direction(n: Vec3, seed: &mut f32) -> Vec3 {
    let r = hash2(seed);
    let temp_vec = if n.y.abs() > 0.5 { Vec3::X } else { Vec3::Y };
    let uu = n.cross(temp_vec).normalize();
    let vv = uu.cross(n);
    let ra = r.y.sqrt();
    let rx = ra * (6.28318530718 * r.x).cos();
    let ry = ra * (6.28318530718 * r.x).sin();
    let rz = (1.0 - r.y).sqrt();
    let rr = rx * uu + ry * vv + rz * n;
    rr.normalize()
}

fn modify_direction_with_roughness(normal: Vec3, n: Vec3, roughness: f32, seed: &mut f32) -> Vec3 {
    let r = hash2(seed);
    let temp_vec = if n.y.abs() > 0.5 { Vec3::X } else { Vec3::Y };
    let uu = n.cross(temp_vec).normalize();
    let vv = uu.cross(n);

    let a = roughness * roughness;
    let rz = ((1.0 - r.y) / (1.0 + (a - 1.0) * r.y).clamp(0.00001, 1.0))
        .abs()
        .sqrt();
    let ra = (1.0 - rz * rz).abs().sqrt();
    let rx = ra * (6.28318530718 * r.x).cos();
    let ry = ra * (6.28318530718 * r.x).sin();
    let rr = rx * uu + ry * vv + rz * n;

    let ret = rr.normalize();
    if ret.dot(normal) > 0.0 {
        ret
    } else {
        (ret + 2.0 * ret.dot(normal) * n).normalize()
    }
}

fn random_in_unit_disk(seed: &mut f32) -> Vec2 {
    let h = hash2(seed) * Vec2::new(1.0, 6.28318530718);
    let phi = h.y;
    let r = h.x.sqrt();
    r * Vec2::new(phi.sin(), phi.cos())
}

fn get_sky_color(rd: Vec3) -> Vec3 {
    let amb = 6.0 - 2.0 * rd.y;
    let sun_dir = Vec3::new(0.4, 0.7, 1.2).normalize();
    let sun = rd.dot(sun_dir).clamp(0.0, 1.0);
    let sun = sun.powf(4.0) + 20.0 * sun.powf(32.0);

    let color_vec = Vec3::new(0.0, 0.6, 1.2);
    let cos_val = Vec3::splat(12.5663706144 * COLOR_SCALE + COLOR_OFFSET) + color_vec;
    let cos_res = Vec3::new(cos_val.x.cos(), cos_val.y.cos(), cos_val.z.cos());

    (Vec3::splat(0.6) + Vec3::splat(0.4) * cos_res) * sun + Vec3::splat(amb)
}

fn render(mut ro: Vec3, mut rd: Vec3, time: f32, seed: &mut f32, mat_hash: &mut f32) -> Vec3 {
    let mut col = Vec3::ZERO;
    let mut emitted = Vec3::ZERO;

    for i in 0..PATH_LENGTH {
        let mut normal = Vec3::ZERO;
        let res = world_hit(
            ro,
            rd,
            Vec2::new(0.0001, MAX_DIST),
            &mut normal,
            time,
            mat_hash,
        );

        if res.y < MAX_DIST {
            ro += rd * res.y;

            let mut albedo = Vec3::ZERO;
            let mut emit = Vec3::ZERO;
            let mut roughness = 0.0;

            if ro.y < -1.34 {
                let ro_xz_len = Vec2::new(ro.x, ro.z).length();
                let albedo_val = FLOOR_BRIGHTNESS / (1.0 + 2.0 * ro_xz_len);
                albedo = Vec3::splat(albedo_val);

                if FLOOR_TYPE < 1.5 {
                    roughness = FLOOR_ROUGHNESS;
                } else {
                    let rotated_xz = rotate_2d(Vec2::new(ro.x, ro.z), std::f32::consts::PI * 0.25);
                    roughness =
                        FLOOR_ROUGHNESS + (4.0 * FLOOR_ROUGHNESS * checker_board(rotated_xz));
                }

                if FLOOR_TYPE < 0.5 {
                    let ws = 0.15 * (ro_xz_len * 20.0 - time * 4.0).sin();
                    let wc = 0.15 * (ro_xz_len * 20.0 - time * 4.0).sin();
                    normal = Vec3::new(ws, 10.0 - ws - wc, wc).normalize();
                }
            } else {
                if MAT_TYPE < 0.5 {
                    let cos_val =
                        Vec3::splat(*mat_hash * (12.5663706144 * COLOR_SCALE) + COLOR_OFFSET)
                            + Vec3::new(0.0, 0.6, 1.2);
                    let cos_res = Vec3::new(cos_val.x.cos(), cos_val.y.cos(), cos_val.z.cos());
                    albedo = Vec3::splat(0.5) + Vec3::splat(0.4) * cos_res;
                    albedo *= albedo;
                } else {
                    let cos_val = Vec3::splat(12.5663706144 * COLOR_SCALE + COLOR_OFFSET)
                        + Vec3::new(0.0, 0.6, 1.2);
                    let cos_res = Vec3::new(cos_val.x.cos(), cos_val.y.cos(), cos_val.z.cos());
                    albedo = Vec3::splat(0.4) + Vec3::splat(0.3) * cos_res;
                    albedo *= albedo;
                    roughness = *mat_hash * *mat_hash * MAT_ROUGHNESS;
                }

                if hash1(*mat_hash) > LIGHT_FILL {
                    let cos_val = Vec3::splat(12.5663706144 * COLOR_SCALE + COLOR_OFFSET)
                        + Vec3::new(0.0, 0.6, 1.2);
                    let cos_res = Vec3::new(cos_val.x.cos(), cos_val.y.cos(), cos_val.z.cos());
                    emit = (Vec3::splat(0.5) + Vec3::splat(0.4) * cos_res) * 50.0;
                }
            }

            if i == 0 {
                emitted += emit;
                col = albedo;
            } else {
                emitted += col * emit;
                col *= albedo;
            }

            if MAT_TYPE < LAMBERTIAN + 0.5 && ro.y > -1.34 {
                rd = cos_weighted_random_hemisphere_direction(normal, seed);
            } else {
                rd = modify_direction_with_roughness(normal, reflect(rd, normal), roughness, seed);
            }
        } else {
            return emitted + col * get_sky_color(rd);
        }
    }
    emitted
}

pub fn fragment_shader(x: u32, y: u32, width: u32, height: u32, time: f32) -> Pixel {
    let fpd = 5.0;
    let mut accum_color = Vec3::ZERO;

    for i in 0..SAMPLES_PER_PIXEL {
        let mut seed = time + 100.0 * ((i as f32) * 2.399 + hash1(x as f32) + hash1(-(y as f32)));

        let mut p = (2.0 * Vec2::new(x as f32, y as f32) - Vec2::new(width as f32, height as f32))
            / (height as f32);

        p.y = -p.y;

        let jitter = hash2(&mut seed);
        let aa_offset = Vec2::new(jitter.x, -jitter.y);
        p += 2.0 * aa_offset / (height as f32);

        let ro_start = Vec3::new(0.0, 0.25, 6.0);
        let rd_start = Vec3::new(p.x, p.y, -2.5).normalize();

        let fp = ro_start + rd_start * fpd;
        let disk = random_in_unit_disk(&mut seed);
        let ro = ro_start + Vec3::new(disk.x, disk.y, 0.0) * 0.05;
        let rd = (fp - ro).normalize();

        let mut mat_hash = 0.0;
        let mut outcol = render(ro, rd, time, &mut seed, &mut mat_hash);

        outcol = (outcol - Vec3::splat(0.004)).max(Vec3::ZERO);
        let num = outcol * (Vec3::splat(6.2) * outcol + Vec3::splat(0.5));
        let den = outcol * (Vec3::splat(6.2) * outcol + Vec3::splat(1.7)) + Vec3::splat(0.06);
        outcol = num / den;

        accum_color += outcol;
    }

    accum_color *= 1.0 / (SAMPLES_PER_PIXEL as f32);

    Pixel {
        r: accum_color.x,
        g: accum_color.y,
        b: accum_color.z,
    }
}

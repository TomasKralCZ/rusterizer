use std::{
    ops::{AddAssign, MulAssign},
    simd::{f32x8, i32x8},
    time::{Duration, Instant},
};

use cgmath::{Deg, Matrix4, Vector2, Vector4};

use crate::{
    camera::Camera,
    raster::Raster,
    solid::{Material, Mesh, Solid},
    HEIGHT, WIDTH,
};

pub struct Renderer {
    raster: Raster,
    camera: Camera,
    persp: Matrix4<f32>,
}

impl Renderer {
    pub fn new(raster: Raster, camera: Camera) -> Self {
        Self {
            raster,
            camera,
            persp: cgmath::perspective(Deg(60.), WIDTH as f32 / HEIGHT as f32, 0.1, 50.),
        }
    }

    pub fn img_buf(&self) -> &[u32] {
        self.raster.img_buf()
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn render_solid(&mut self, solid: &Solid) {
        self.raster.clear();

        let start = Instant::now();

        for mesh in &solid.meshes {
            self.render_mesh(mesh);
        }

        let elapsed = Instant::now().duration_since(start);

        if elapsed > Duration::from_millis(16) {
            println!("{elapsed:?}");
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh) {
        for ([v1pos, v2pos, v3pos], [v1tex, v2tex, v3tex]) in
            mesh.pos_indices.iter().zip(mesh.texcoord_indices.iter())
        {
            let pos = &mesh.positions;
            let tex = &mesh.texcoords;

            let v1 = Vertex::new(pos[*v1pos as usize], tex[*v1tex as usize]);
            let v2 = Vertex::new(pos[*v2pos as usize], tex[*v2tex as usize]);
            let v3 = Vertex::new(pos[*v3pos as usize], tex[*v3tex as usize]);

            self.transform_triangle(v1, v2, v3, &mesh.material);
        }
    }

    fn transform_triangle(
        &mut self,
        mut v1: Vertex,
        mut v2: Vertex,
        mut v3: Vertex,
        mat: &Material,
    ) {
        let model = Matrix4::from_angle_y(Deg(270.));
        let view = self.camera.get_view_mat();

        let transforms = self.persp * view * model;

        v1.transform(transforms);
        v2.transform(transforms);
        v3.transform(transforms);

        if [&v1, &v2, &v3].iter().all(|v| v.pos.z <= 0.)
            || [&v1, &v2, &v3].iter().all(|v| v.pos.z > v.pos.w)
            || [&v1, &v2, &v3].iter().all(|v| v.pos.y > v.pos.w)
            || [&v1, &v2, &v3].iter().all(|v| v.pos.y < -v.pos.w)
            || [&v1, &v2, &v3].iter().all(|v| v.pos.x > v.pos.w)
            || [&v1, &v2, &v3].iter().all(|v| v.pos.x < -v.pos.w)
        {
            return;
        }

        v1.dehomog();
        v2.dehomog();
        v3.dehomog();

        self.render_triangle(v1, v2, v3, mat);
    }

    fn render_triangle(&mut self, mut v1: Vertex, mut v2: Vertex, mut v3: Vertex, mat: &Material) {
        let v1c = v1.to_screen_coords();
        let v2c = v2.to_screen_coords();
        let v3c = v3.to_screen_coords();

        let minx = [v1c, v2c, v3c].iter().map(|v| v.x).min().unwrap();
        let maxx = [v1c, v2c, v3c].iter().map(|v| v.x).max().unwrap();
        let miny = [v1c, v2c, v3c].iter().map(|v| v.y).min().unwrap();
        let maxy = [v1c, v2c, v3c].iter().map(|v| v.y).max().unwrap();

        let area = (v2c.x as f32 - v1c.x as f32) * (v3c.y as f32 - v1c.y as f32)
            - (v3c.x as f32 - v1c.x as f32) * (v2c.y as f32 - v1c.y as f32);

        for y in miny..maxy {
            for x in minx..maxx {
                self.draw_pixel(x, y, v1c, v2c, v3c, &v1, &v2, &v3, area as f32, mat);
            }
        }

        /* for y in miny..=maxy {
            for x in (minx..=maxx).step_by(8) {
                /* if x + 7 > maxx {
                    for x in x..=maxx {
                        self.draw_pixel(x, y, v1c, v2c, v3c, &v1, &v2, &v3, area as f32, mat);
                    }
                } else { */
                let xs = i32x8::from_array([x, x + 1, x + 2, x + 3, x + 4, x + 5, x + 6, x + 7]);
                self.draw_pixel_simd(xs, y, v1c, v2c, v3c, &v1, &v2, &v3, area as f32, mat);
                /* } */
            }
        } */
    }

    /* fn draw_pixel_simd(
        &mut self,
        xs: i32x8,
        y: i32,
        v1c: Vector2<i32>,
        v2c: Vector2<i32>,
        v3c: Vector2<i32>,
        v1: &Vertex,
        v2: &Vertex,
        v3: &Vertex,
        total_area: f32,
        mat: &Material,
    ) {
        let (v1t, v2t, v3t) = Self::barycentric_simd(xs, y, v1c, v2c, v3c, total_area);
        for i in 0..8 {
            if v1t[i] >= 0. && v2t[i] >= 0. && v3t[i] >= 0. {
                let v = Vertex::lerp(v1, v2, v3, v1t[i], v2t[i], v3t[i]);
                let z = v.pos.z;
                let tex_color = Self::sample_texture(v, mat);
                self.raster
                    .set_pixel(xs[i] as usize, y as usize, tex_color, z);
            }
        }
    } */

    fn draw_pixel(
        &mut self,
        x: i32,
        y: i32,
        v1c: Vector2<i32>,
        v2c: Vector2<i32>,
        v3c: Vector2<i32>,
        v1: &Vertex,
        v2: &Vertex,
        v3: &Vertex,
        total_area: f32,
        mat: &Material,
    ) {
        let (v1t, v2t, v3t) = Self::barycentric(x, y, v1c, v2c, v3c, total_area);
        if v1t >= 0. && v2t >= 0. && v3t >= 0. {
            let v = Vertex::lerp(v1, v2, v3, v1t, v2t, v3t);
            let z = v.pos.z;
            let tex_color = Self::sample_texture(v, mat);
            self.raster.set_pixel(x as usize, y as usize, tex_color, z);
        }
    }

    /* fn barycentric_simd(
        xs: i32x8,
        y: i32,
        v1c: Vector2<i32>,
        v2c: Vector2<i32>,
        v3c: Vector2<i32>,
        total_area: f32,
    ) -> (f32x8, f32x8, f32x8) {
        let c_sub_area = i32x8::splat(v2c.x - v1c.x) * i32x8::splat(y - v1c.y)
            - (xs - i32x8::splat(v1c.x)) * i32x8::splat(v2c.y - v1c.y);
        let b_sub_area = i32x8::splat(v1c.x - v3c.x) * i32x8::splat(y - v3c.y)
            - (xs - i32x8::splat(v3c.x)) * i32x8::splat(v1c.y - v3c.y);

        let ct = c_sub_area.cast::<f32>() / f32x8::splat(total_area);
        let bt = b_sub_area.cast::<f32>() / f32x8::splat(total_area);
        let at = f32x8::splat(1.) - (ct + bt);

        (at, bt, ct)
    } */

    fn barycentric(
        x: i32,
        y: i32,
        v1c: Vector2<i32>,
        v2c: Vector2<i32>,
        v3c: Vector2<i32>,
        total_area: f32,
    ) -> (f32, f32, f32) {
        let c_sub_area = (v2c.x - v1c.x) * (y - v1c.y) - (x - v1c.x) * (v2c.y - v1c.y);
        let b_sub_area = (v1c.x - v3c.x) * (y - v3c.y) - (x - v3c.x) * (v1c.y - v3c.y);

        let ct = c_sub_area as f32 / total_area;
        let bt = b_sub_area as f32 / total_area;
        let at = 1. - (ct + bt);

        (at, bt, ct)
    }

    fn sample_texture(v: Vertex, mat: &Material) -> u32 {
        let tex = &mat.diffuse_texture;
        let tx = v.texcoords().x * tex.width as f32;
        let ty = v.texcoords().y * tex.height as f32;

        let col = tex.get_pixel(tx as usize, tex.height as usize - 1 - ty as usize);
        u32::from_be_bytes([0, col.x, col.y, col.z])

        // https://en.wikipedia.org/wiki/Bilinear_interpolation#Weighted_mean
        /* let x1 = tx.round();
        let x2 = (tx + 1.).round();
        let y1 = (ty + 1.).round();
        let y2 = ty.round();

        let rect_area = (x2 - x1) * (y2 - y1);
        let w_bot_l = ((x2 - tx) * (y2 - ty)) / rect_area;
        let w_top_l = ((x2 - tx) * (ty - y1)) / rect_area;
        let w_bot_r = ((tx - x1) * (y2 - ty)) / rect_area;
        let w_top_r = ((tx - x1) * (ty - y1)) / rect_area;

        let mut top_lc = tex
            .get_pixel(
                (x1 as u32).clamp(0, tex.width - 1) as usize,
                (tex.height - (y2 as u32).clamp(0, tex.height - 1)) as usize,
            )
            .cast::<f32>()
            .unwrap();
        let mut top_rc = tex
            .get_pixel(
                (x2 as u32).clamp(0, tex.width - 1) as usize,
                (tex.height - (y2 as u32).clamp(0, tex.height - 1)) as usize,
            )
            .cast::<f32>()
            .unwrap();
        let mut bot_lc = tex
            .get_pixel(
                (x1 as u32).clamp(0, tex.width - 1) as usize,
                (tex.height - (y1 as u32).clamp(0, tex.height - 1)) as usize,
            )
            .cast::<f32>()
            .unwrap();
        let mut bot_rc = tex
            .get_pixel(
                (x2 as u32).clamp(0, tex.width - 1) as usize,
                (tex.height - (y1 as u32).clamp(0, tex.height - 1)) as usize,
            )
            .cast::<f32>()
            .unwrap();

        top_lc *= w_top_l;
        top_rc *= w_top_r;
        bot_lc *= w_bot_l;
        bot_rc *= w_bot_r;

        let res_c = top_lc + top_rc + bot_lc + bot_rc;
        u32::from_be_bytes([0, res_c[0] as u8, res_c[1] as u8, res_c[2] as u8]) */
    }
}

#[derive(Debug, Clone)]
struct Vertex {
    pos: Vector4<f32>,
    texcoords: Vector2<f32>,
    one: f32,
}

impl Vertex {
    fn new(pos: Vector4<f32>, texcoords: Vector2<f32>) -> Self {
        Self {
            pos,
            texcoords,
            one: 1.,
        }
    }

    fn transform(&mut self, mat: Matrix4<f32>) {
        self.pos = mat * self.pos;
    }

    fn dehomog(&mut self) {
        let w = self.pos.w;
        self.pos /= w;
        self.texcoords /= w;
        self.one /= w;
    }

    fn texcoords(&self) -> Vector2<f32> {
        self.texcoords * (1. / self.one)
    }

    fn to_screen_coords(&mut self) -> Vector2<i32> {
        let x = 0.5 * (WIDTH - 1) as f32 * (self.pos.x + 1.);
        let y = 0.5 * (HEIGHT - 1) as f32 * (1. - self.pos.y);

        Vector2::new(x as i32, y as i32)
    }

    fn lerp(v1: &Vertex, v2: &Vertex, v3: &Vertex, v1t: f32, v2t: f32, v3t: f32) -> Vertex {
        let mut v1 = v1.clone();
        v1 *= v1t;

        let mut v2 = v2.clone();
        v2 *= v2t;

        let mut v3 = v3.clone();
        v3 *= v3t;

        v1 += v2;
        v1 += v3;

        v1
    }
}

impl MulAssign<f32> for Vertex {
    fn mul_assign(&mut self, rhs: f32) {
        self.pos *= rhs;
        self.texcoords *= rhs;
        self.one *= rhs;
    }
}

impl AddAssign<Vertex> for Vertex {
    fn add_assign(&mut self, rhs: Vertex) {
        self.pos += rhs.pos;
        self.texcoords += rhs.texcoords;
        self.one += rhs.one;
    }
}

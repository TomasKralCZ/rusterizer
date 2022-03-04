use cgmath::{Point3, Vector2, Vector3, Vector4};
use image::DynamicImage;

pub struct Solid {
    pub meshes: Vec<Mesh>,
}

impl Solid {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Solid { meshes }
    }
}

pub struct Mesh {
    pub positions: Vec<Vector4<f32>>,
    pub pos_indices: Vec<[u32; 3]>,

    pub normals: Vec<Vector3<f32>>,
    pub normal_indices: Vec<[u32; 3]>,

    pub texcoords: Vec<Vector2<f32>>,
    pub texcoord_indices: Vec<[u32; 3]>,

    pub material: Material,
}

impl Mesh {
    pub fn new(
        positions: Vec<f32>,
        pos_indices: Vec<u32>,
        normals: Vec<f32>,
        normal_indices: Vec<u32>,
        texcoords: Vec<f32>,
        texcoord_indices: Vec<u32>,
        material: Material,
    ) -> Self {
        let positions = positions
            .array_chunks()
            .map(|c| Point3::from(*c).to_homogeneous())
            .collect();
        let pos_indices = pos_indices.array_chunks().map(|c| *c).collect();

        let normals = normals.array_chunks().map(|c| Vector3::from(*c)).collect();
        let normal_indices = normal_indices.array_chunks().map(|c| *c).collect();

        let texcoords = texcoords
            .array_chunks()
            .map(|c| Vector2::from(*c))
            .collect();
        let texcoord_indices = texcoord_indices.array_chunks().map(|c| *c).collect();

        Self {
            positions,
            pos_indices,
            normals,
            normal_indices,
            texcoords,
            texcoord_indices,
            material,
        }
    }
}

pub struct Material {
    // ambient
    // diffuse
    // specular
    // shininess
    pub diffuse_texture: Texture,
}

impl Material {
    pub fn new(diffuse_texture: DynamicImage) -> Self {
        let diffuse_texture = diffuse_texture.into_rgba8();
        let flat = diffuse_texture
            .as_raw()
            .array_chunks::<4>()
            .map(|c| Vector3::new(c[0], c[1], c[2]))
            .collect();

        Self {
            diffuse_texture: Texture::new(flat, diffuse_texture.width(), diffuse_texture.height()),
        }
    }
}

pub struct Texture {
    pixels: Vec<Vector3<u8>>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(pixels: Vec<Vector3<u8>>, width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vector3<u8> {
        let index = y * self.width as usize + x;

        if index >= self.pixels.len() {
            return Vector3::new(255, 255, 255);
        }

        self.pixels[index]
    }
}

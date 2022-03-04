use eyre::Result;

use crate::solid::{Material, Mesh, Solid};

pub fn load_solid(obj_path: &str, tex_dir: &str) -> Result<Solid> {
    let mut load_options = tobj::LoadOptions::default();
    load_options.triangulate = true;

    let (models, materials) = tobj::load_obj(&obj_path, &load_options)?;
    let materials = materials?;

    let mut meshes = Vec::new();
    for model in models {
        // TODO: modely bet materiálů...
        let mat_index = model.mesh.material_id.unwrap();
        let material = &materials[mat_index];
        let diffuse_texture_path = format!("{}{}", tex_dir, material.diffuse_texture);

        let diffuse_texture = if material.diffuse_texture == "" {
            image::DynamicImage::new_rgb8(1, 1)
        } else {
            image::open(diffuse_texture_path)?
        };

        let material = Material::new(diffuse_texture);

        let mesh = Mesh::new(
            model.mesh.positions,
            model.mesh.indices,
            model.mesh.normals,
            model.mesh.normal_indices,
            model.mesh.texcoords,
            model.mesh.texcoord_indices,
            material,
        );

        meshes.push(mesh);
    }

    Ok(Solid::new(meshes))
}

use std::{path::Path, rc::Rc};
use cgmath::{vec2, vec3, Matrix4};
use super::{ShaderProgram, Mesh, Vertex, Texture, GlError};

#[derive(Default)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    // Stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    // Reference counter to ensure textures are dropped properly.
    pub textures_loaded: Vec<Rc<Texture>>,
    directory: String
}

impl Model {
    pub fn new(path: &str, model_transforms: Vec<Matrix4<f32>>) -> Result<Model, GlError> {
        let mut model = Model::default();
        model.load_model(path, model_transforms)?;
        Ok(model)
    }

    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        for mesh in &self.meshes {
            mesh.draw(shader_program)?;
        }

        Ok(())
    }

    pub fn load_model(&mut self, path: &str, model_transforms: Vec<Matrix4<f32>>) -> Result<(), GlError> {
        let path = Path::new(path);
        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        
        let obj = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);

        let (models, materials) = obj?;
        let materials = materials?; // Fix broken return type
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            // Data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            // TODO: with https://learnopengl.com/Advanced-OpenGL/Advanced-Data,
            // TODO: it could be possible to store less data on the gpu using uneven data and verteces, as is default
            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);

            for i in 0..num_vertices {
                vertices.push(
                    Vertex {
                        position: vec3(p[i*3], p[i*3+1], p[i*3+2]),
                        normal: vec3(n[i*3], n[i*3+1], n[i*3+2]),
                        tex_coord: vec2(t[i*2], t[i*2+1]),
                        ..Vertex::default()
                    }
                )
            }

            // Process material
            let mut textures: Vec<Rc<Texture>> = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                // Diffuse map
                if !material.diffuse_texture.is_empty() {
                    let texture = self.load_material_texture(&material.diffuse_texture, "diffuse")?;
                    textures.push(texture);
                }
                // Specular map
                if !material.specular_texture.is_empty() {
                    let texture = self.load_material_texture(&material.specular_texture, "specular")?;
                    textures.push(texture);
                }
                // Normal map
                if !material.normal_texture.is_empty() {
                    let texture = self.load_material_texture(&material.normal_texture, "normal")?;
                    textures.push(texture);
                }
            }

            self.meshes.push(Mesh::new(vertices, indices, textures, model_transforms.to_vec()));
        }

        Ok(())
    }

    pub fn load_material_texture(&mut self, path: &str, type_: &str) -> Result<Rc<Texture>, GlError> {
        let texture = self.textures_loaded.iter().find(|t| t.path == path);
        if let Some(texture) = texture {
            return Ok(Rc::clone(texture));
        }

        let path = format!("{}/{}", &self.directory, path);
        let texture = Rc::new(Texture::from_file_2d(&path, type_)?);
        let result = Rc::clone(&texture);

        // Send owned RC to loaded textures, and reference to the actual mesh
        self.textures_loaded.push(texture);
        Ok(result)
    }
}
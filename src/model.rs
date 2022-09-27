use std::path::Path;
use cgmath::{vec2, vec3, Matrix4};
use crate::{mesh::{Mesh, Vertex, Texture}, shader_program::ShaderProgram};
use image::DynamicImage::*;

#[derive(Default)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>, // Stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    directory: String
}

impl Model {
    pub fn new(path: &str, model_transforms: Vec<Matrix4<f32>>) -> Model {
        let mut model = Model::default();
        model.load_model(path, model_transforms);
        model
    }

    pub fn draw(&self, shader_program: &ShaderProgram) {
        for mesh in &self.meshes {
            unsafe { mesh.draw(shader_program) }
        }
    }

    fn load_model(&mut self, path: &str, model_transforms: Vec<Matrix4<f32>>) {
        let path = Path::new(path);
        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        
        let obj = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);

        let (models, materials) = obj.unwrap();
        let materials = materials.unwrap(); // Fix broken return type
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
            let mut textures: Vec<Texture> = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                // Diffuse map
                if !material.diffuse_texture.is_empty() {
                    let texture = self.load_material_texture(&material.diffuse_texture, "diffuse");
                    textures.push(texture);
                }
                // Specular map
                if !material.specular_texture.is_empty() {
                    let texture = self.load_material_texture(&material.specular_texture, "specular");
                    textures.push(texture);
                }
                // Normal map
                if !material.normal_texture.is_empty() {
                    let texture = self.load_material_texture(&material.normal_texture, "normal");
                    textures.push(texture);
                }
            }

            self.meshes.push(Mesh::new(vertices, indices, textures, model_transforms.to_vec()));
        }

        // TODO: maybe clear up memory by clearing textures loaded?
    }

    fn load_material_texture(&mut self, path: &str, type_: &str) -> Texture {
        let texture = self.textures_loaded.iter().find(|t| t.path == path);
        if let Some(texture) = texture {
            return texture.clone();
        }

        let texture = Texture {
            id: unsafe { Model::texture_from_file_split_path(path, &self.directory) },
            type_: type_.into(),
            path: path.into()
        };
        self.textures_loaded.push(texture.clone());
        texture
    }

    pub unsafe fn texture_from_file_split_path(path: &str, directory: &str) -> u32 {
        let filename = format!("{}/{}", directory, path);
        
        Model::texture_from_file(filename.as_str())
    }

    pub unsafe fn texture_from_file(path: &str) -> u32 {
        let mut texture_id = 0;
        gl::GenTextures(1, &mut texture_id);

        let img = image::open(&Path::new(&path)).expect("Failed to load texture");
        // let img = img.flipv();
        let data = img.as_bytes();

        // TODO: if there is an alpha, mark mesh as transparent
        let (internal_format, data_format) = match img {
            ImageLuma8(_) => (gl::RED, gl::RED),
            ImageLumaA8(_) => (gl::RG, gl::RG),
            ImageRgb8(_) => (gl::SRGB, gl::RGB),
            ImageRgba8(_) => (gl::SRGB_ALPHA, gl::RGBA),
            _ => (gl::SRGB, gl::RGB) // If nothing else, try default
        };
    

        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal_format as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            data_format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const gl::types::GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        texture_id
    }
}
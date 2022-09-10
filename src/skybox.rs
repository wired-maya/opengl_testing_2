use std::path::Path;
use cgmath::{Vector3, Vector2};
use image::DynamicImage::*;

use crate::{mesh::{Texture, Mesh, Vertex}, shader_program::ShaderProgram};

pub struct Skybox {
    pub mesh: Mesh,
    faces: Vec<String>
}

impl Skybox {
    pub fn new(faces: Vec<String>) -> Skybox {
        // Create mesh for skybox
        let texture = Texture {
            id: 0,
            type_: "diffuse".to_owned(),
            path: "".to_owned()
        };

        // Cube definition
        let vertices = vec![
            Vertex {
                position: Vector3::new(-1.0, 1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(-1.0, 1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
        ];

        let indices = vec![
            0, 2, 3, 3, 1, 0,
            6, 2, 0, 0, 4, 6,
            3, 7, 5, 5, 1, 3,
            6, 4, 5, 5, 7, 6,
            0, 1, 5, 5, 4, 0,
            2, 6, 3, 3, 6, 7
        ];

        let textures = vec![
            texture
        ];

        let mesh = Mesh::new(vertices, indices, textures);

        let mut skybox = Skybox {
            mesh,
            faces
        };

        unsafe { skybox.load_cube_map() }

        skybox
    }

    unsafe fn load_cube_map(&mut self) {
        gl::GenTextures(1, &mut self.mesh.textures[0].id);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.mesh.textures[0].id);

        for (i, face) in self.faces.iter().enumerate() {
            let img = image::open(&Path::new(face)).expect("Failed to load cubemap texture");
            let data = img.as_bytes();

            let format = match img {
                ImageLuma8(_) => gl::RED,
                ImageLumaA8(_) => gl::RG,
                ImageRgb8(_) => gl::RGB,
                ImageRgba8(_) => gl::RGBA,
                _ => todo!()
            };

            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                0,
                format as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const gl::types::GLvoid
            );
        }

        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
    }

    pub unsafe fn draw(&self, shader_program: &ShaderProgram) {
        // Change depth func so test values pass when they are equal to the buffer's content
        gl::DepthFunc(gl::LEQUAL);

        shader_program.use_program();
        self.mesh.draw(shader_program);

        gl::DepthFunc(gl::LESS);
    }
}
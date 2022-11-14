use std::rc::Rc;

use cgmath::{Vector3, Vector2, Matrix4, vec3};
use super::{Texture, Mesh, Vertex, ShaderProgram, GlError};

pub struct Skybox {
    pub mesh: Mesh
}

impl Skybox {
    pub fn new(faces: Vec<String>) -> Result<Skybox, GlError> {
        // Cube definition
        let vertices = vec![
            Vertex {
                position: Vector3::new(-1.0, 1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, -1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(-1.0, 1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, 1.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
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
            Rc::new(Texture::from_file_cubemap(faces)?)
        ];

        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];

        let mesh = Mesh::new(vertices, indices, textures, model_transforms);

        let skybox = Skybox { mesh };

        Ok(skybox)
    }

    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        unsafe {
            // Change depth func so test values pass when they are equal to the buffer's content
            gl::DepthFunc(gl::LEQUAL);

            self.mesh.draw(shader_program)?;

            gl::DepthFunc(gl::LESS);
        }

        Ok(())
    }
}
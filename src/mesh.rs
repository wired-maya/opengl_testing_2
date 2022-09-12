use cgmath::{Vector3, Vector2, Zero};
use memoffset::offset_of;

use crate::shader_program::ShaderProgram;

// TODO: sort any meshes with alpha values and render them farthest to closest w/o depth buffer
// TODO: make sure to move vertex and texture structs to their own files

#[repr(C, packed)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coord: Vector2::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub type_: String,
    pub path: String
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    vao: u32,
    vbo: u32,
    ebo: u32
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices, indices, textures,
            vao: 0, vbo: 0, ebo: 0
        };

        unsafe { mesh.setup_mesh() }
        mesh
    }

    unsafe fn setup_mesh(&mut self) {
        gl::GenVertexArrays(1, &mut self.vao);
        gl::GenBuffers(1, &mut self.vbo);
        gl::GenBuffers(1, &mut self.ebo);

        gl::BindVertexArray(self.vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (self.vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
            self.vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (self.indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
            self.indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        let stride = std::mem::size_of::<Vertex>() as gl::types::GLint;

        // Vertex positions
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset_of!(Vertex, position) as *const gl::types::GLvoid
        );
        // Vertex normals
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset_of!(Vertex, normal) as *const gl::types::GLvoid
        );
        // Vertex texture coords
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset_of!(Vertex, tex_coord) as *const gl::types::GLvoid
        );

        gl::BindVertexArray(0);
    }

    pub unsafe fn draw(&self, shader_program: &ShaderProgram) {
        let mut diffuse_num: u32 = 0;
        let mut specular_num: u32 = 0;

        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            let name = &texture.type_;
            let _number = match name.as_str() {
                "diffuse" => {
                    diffuse_num += 1;
                    diffuse_num
                }
                "specular" => {
                    specular_num += 1;
                    specular_num
                }
                _ => panic!("unknown texture type")
            };

            // shader_program.set_int(format!("material.{}{}", name, number).as_str(), i as i32);
            // Ignores numbers for now
            shader_program.set_int(format!("material.{}", name).as_str(), i as i32);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        // Draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
        gl::BindVertexArray(0);

        // Set back to defaults once configured
        gl::ActiveTexture(gl::TEXTURE0);
    }

    pub unsafe fn _draw_instanced(&self, shader_program: &ShaderProgram, instancecount: i32) {
        let mut diffuse_num: u32 = 0;
        let mut specular_num: u32 = 0;

        for (i, texture) in self.textures.iter().enumerate() {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            let name = &texture.type_;
            let _number = match name.as_str() {
                "diffuse" => {
                    diffuse_num += 1;
                    diffuse_num
                }
                "specular" => {
                    specular_num += 1;
                    specular_num
                }
                _ => panic!("unknown texture type")
            };

            // shader_program.set_int(format!("material.{}{}", name, number).as_str(), i as i32);
            // Ignores numbers for now
            shader_program.set_int(format!("material.{}", name).as_str(), i as i32);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        // Draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElementsInstanced(
            gl::TRIANGLES,
            self.indices.len() as i32,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            instancecount
        );
        gl::BindVertexArray(0);

        // Set back to defaults once configured
        gl::ActiveTexture(gl::TEXTURE0);
    }
}
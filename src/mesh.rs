use cgmath::{Vector3, Vector2, Zero, Matrix4, Vector4};
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
    pub model_transforms: Vec<Matrix4<f32>>,
    pub vao: u32,
    vbo: u32,
    ebo: u32,
    tbo: u32
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>, model_transforms: Vec<Matrix4<f32>>) -> Mesh {
        let mut mesh = Mesh {
            vertices, indices, textures, model_transforms,
            vao: 0, vbo: 0, ebo: 0, tbo: 0
        };

        unsafe { 
            mesh.setup_mesh();
            mesh.setup_transform_attribute();
        }
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

    unsafe fn setup_transform_attribute(&mut self) {
        let size_mat4 = std::mem::size_of::<Matrix4<f32>>() as i32;
        let size_vec4 = std::mem::size_of::<Vector4<f32>>() as i32;

        // Bind transforms so they are sent to the shader program
        gl::GenBuffers(1, &mut self.tbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.tbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (self.model_transforms.len() as i32 * size_mat4) as isize,
            self.model_transforms.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        gl::BindVertexArray(self.vao);

        // Max data size is vec4, so send mat4 as 4 vec4s
        for i in 0..4 {
            gl::EnableVertexAttribArray(3 + i);
            gl::VertexAttribPointer(
                3 + i,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_mat4,
                (i as i32 * size_vec4) as *const gl::types::GLvoid
            );
            gl::VertexAttribDivisor(3 + i, 1);
        }

        gl::BindVertexArray(0);
    }

    pub unsafe fn reset_material(shader_program: &ShaderProgram) {
        shader_program.set_bool("material.has_diffuse", false);
        shader_program.set_bool("material.has_specular", false);
    }

    pub unsafe fn draw(&self, shader_program: &ShaderProgram) {
        let mut diffuse_num: u32 = 0;
        let mut specular_num: u32 = 0;

        Mesh::reset_material(shader_program);

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
            // TODO: make this a texture array ig? Ignores numbers for now
            shader_program.set_int(format!("material.{}", name).as_str(), i as i32);
            shader_program.set_bool(format!("material.has_{}", name).as_str(), true);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }

        // Draw mesh
        gl::BindVertexArray(self.vao);
        gl::DrawElementsInstanced(
            gl::TRIANGLES,
            self.indices.len() as i32,
            gl::UNSIGNED_INT,
            std::ptr::null(),
            self.model_transforms.len() as i32
        );
        gl::BindVertexArray(0);

        // Set back to defaults once configured
        gl::ActiveTexture(gl::TEXTURE0);
    }
}
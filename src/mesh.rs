use cgmath::{Vector3, Vector2, Zero, Matrix4, Vector4, InnerSpace, vec2};
use memoffset::offset_of;

use crate::shader_program::ShaderProgram;

// TODO: sort any meshes with alpha values and render them farthest to closest w/o depth buffer
// TODO: make sure to move vertex and texture structs to their own files

#[repr(C, packed)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub tangent: Vector3<f32>,
    pub bitangent: Vector3<f32>
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coord: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero()
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
            mesh.calc_vertex_tangents();
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
        // Vertex tangent
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(
            3,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset_of!(Vertex, tangent) as *const gl::types::GLvoid
        );
        // Vertex bitangent
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(
            4,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            offset_of!(Vertex, bitangent) as *const gl::types::GLvoid
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
            gl::EnableVertexAttribArray(5 + i);
            gl::VertexAttribPointer(
                5 + i,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_mat4,
                (i as i32 * size_vec4) as *const gl::types::GLvoid
            );
            gl::VertexAttribDivisor(5 + i, 1);
        }

        gl::BindVertexArray(0);
    }

    // Calculate all tangent vectors for the vertices for lighting
    pub unsafe fn calc_vertex_tangents(&mut self) {
        for i in 0..(self.indices.len() / 3) {
            let index = i * 3;

            let index1 = self.indices[index] as usize;
            let index2 = self.indices[index + 1] as usize;
            let index3 = self.indices[index + 2] as usize;

            // Get positions for the vertices that make up the triangle
            let pos1 = self.vertices[index1].position;
            let pos2 = self.vertices[index2].position;
            let pos3 = self.vertices[index3].position;

            // Get corresponding texture coordinates
            let uv1 = self.vertices[index1].tex_coord;
            let uv2 = self.vertices[index2].tex_coord;
            let uv3 = self.vertices[index3].tex_coord;

            // Calculate deltas
            let edge1 = pos2 - pos1;
            let edge2 = pos3 - pos1;
            let mut delta_uv1 = uv2 - uv1;
            let mut delta_uv2 = uv3 - uv1;

            // let f: f32 = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);
            let dir_correction: bool = (delta_uv2.x * delta_uv1.y - delta_uv2.y * delta_uv1.x) < 0.0;
            let dir_correction: f32 = if dir_correction { -1.0 } else { 1.0 };

            if delta_uv1.x * delta_uv2.y == delta_uv1.y * delta_uv2.x {
                delta_uv1 = vec2(0.0, 1.0);
                delta_uv2 = vec2(1.0, 0.0);
            }

            // Create tangent vector
            let mut tangent: Vector3<f32> = Vector3::zero();
            let mut bitangent: Vector3<f32> = Vector3::zero();

            // Calculate tangent vector
            // tangent.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
            // tangent.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
            // tangent.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);

            // bitangent.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
            // bitangent.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
            // bitangent.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);

            tangent.x = dir_correction * (edge2.x * delta_uv1.y - edge1.x * delta_uv2.y);
            tangent.y = dir_correction * (edge2.y * delta_uv1.y - edge1.y * delta_uv2.y);
            tangent.z = dir_correction * (edge2.z * delta_uv1.y - edge1.z * delta_uv2.y);
            
            bitangent.x = dir_correction * ( - edge2.x * delta_uv1.x + edge1.x * delta_uv2.x);
            bitangent.y = dir_correction * ( - edge2.y * delta_uv1.x + edge1.y * delta_uv2.x);
            bitangent.z = dir_correction * ( - edge2.z * delta_uv1.x + edge1.z * delta_uv2.x);

            // tangent = tangent.normalize();
            // bitangent = bitangent.normalize();

            // Set tangent vector to all vertices of the triangle
            self.vertices[index1].tangent = tangent;
            self.vertices[index2].tangent = tangent;
            self.vertices[index3].tangent = tangent;            

            self.vertices[index1].bitangent = bitangent;
            self.vertices[index2].bitangent = bitangent;
            self.vertices[index3].bitangent = bitangent;

            // for i in 0..3 {
            //     let local_index = self.indices[index + i] as usize;
            //     let local_normal = self.vertices[local_index].normal;
            //     // TODO: could be dot product
            //     let mut local_tangent = tangent - local_normal.cross(tangent.cross(local_normal));
            //     let mut local_bitangent = bitangent - local_normal.cross(bitangent.cross(local_normal)) - local_tangent.cross(bitangent.cross(local_tangent));

            //     local_tangent = local_tangent.normalize();
            //     local_bitangent = local_bitangent.normalize();

            //     self.vertices[local_index].tangent = local_tangent;
            //     self.vertices[local_index].bitangent = local_bitangent;
            // }
        }
    }

    pub unsafe fn reset_material(shader_program: &ShaderProgram) {
        shader_program.set_bool("material.has_diffuse", false);
        shader_program.set_bool("material.has_specular", false);
        shader_program.set_bool("material.has_normal", false);
    }

    pub unsafe fn draw(&self, shader_program: &ShaderProgram) {
        let mut diffuse_num: u32 = 0;
        let mut specular_num: u32 = 0;
        let mut normal_num: u32 = 0;

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
                "normal" => {
                    normal_num += 1;
                    normal_num
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
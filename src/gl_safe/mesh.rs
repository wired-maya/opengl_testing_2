use std::rc::Rc;

use cgmath::{Vector3, Zero, Matrix4, vec2};
use memoffset::offset_of;
use super::{ShaderProgram, GlError, VertexArray, Buffer, Texture, Vertex};

// TODO: sort any meshes with alpha values and render them farthest to closest w/o depth buffer

pub struct Mesh {
    pub textures: Vec<Rc<Texture>>, // TODO: Use data types with less overhead, like array instead of Vec<> (though do research if indexing is really slower)
    pub vao: VertexArray,
    pub vbo: Buffer<Vertex>,
    pub ebo: Buffer<u32>,
    pub tbo: Buffer<Matrix4<f32>>
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Rc<Texture>>, model_transforms: Vec<Matrix4<f32>>) -> Mesh {
        let mut mesh = Mesh {
            textures,
            vao: VertexArray::new(),
            vbo: Buffer::new(),
            ebo: Buffer::new(),
            tbo: Buffer::new()
        };

        // Move data to mutable variable
        let (mut vertices, mut indices) = (vertices, indices);

        mesh.calc_vertex_tangents(&mut vertices, &mut indices); // Operates on data before it's sent to buffers
        mesh.setup_mesh(vertices, indices);
        mesh.setup_transform_attribute(model_transforms);

        mesh
    }

    fn setup_mesh(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>) {
        self.vbo.send_data_static(&self.vao, vertices, gl::ARRAY_BUFFER);
        self.ebo.send_data_static(&self.vao, indices, gl::ELEMENT_ARRAY_BUFFER);

        let stride = std::mem::size_of::<Vertex>() as gl::types::GLint;

        self.vao.add_attrib(3, stride, offset_of!(Vertex, position) as *const gl::types::GLvoid);
        self.vao.add_attrib(3, stride, offset_of!(Vertex, normal) as *const gl::types::GLvoid);
        self.vao.add_attrib(2, stride, offset_of!(Vertex, tex_coord) as *const gl::types::GLvoid);
        self.vao.add_attrib(3, stride, offset_of!(Vertex, tangent) as *const gl::types::GLvoid);
        self.vao.add_attrib(3, stride, offset_of!(Vertex, bitangent) as *const gl::types::GLvoid);
    }

    fn setup_transform_attribute(&mut self, model_transforms: Vec<Matrix4<f32>>) {
        self.tbo.send_data_static(&self.vao, model_transforms, gl::ARRAY_BUFFER);
        self.vao.add_attrib_divisor(4);
    }

    pub fn calc_vertex_tangents(&mut self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        for i in 0..(indices.len() / 3) {
            let index = i * 3;

            let index1 = indices[index] as usize;
            let index2 = indices[index + 1] as usize;
            let index3 = indices[index + 2] as usize;

            // Get positions for the vertices that make up the triangle
            let pos1 = vertices[index1].position;
            let pos2 = vertices[index2].position;
            let pos3 = vertices[index3].position;

            // Get corresponding texture coordinates
            let uv1 = vertices[index1].tex_coord;
            let uv2 = vertices[index2].tex_coord;
            let uv3 = vertices[index3].tex_coord;

            // Calculate deltas
            let edge1 = pos2 - pos1;
            let edge2 = pos3 - pos1;
            let mut delta_uv1 = uv2 - uv1;
            let mut delta_uv2 = uv3 - uv1;

            // Slight correction for angles to be more accurate
            let dir_correction: bool = (delta_uv2.x * delta_uv1.y - delta_uv2.y * delta_uv1.x) < 0.0;
            let dir_correction: f32 = if dir_correction { -1.0 } else { 1.0 };

            if delta_uv1.x * delta_uv2.y == delta_uv1.y * delta_uv2.x {
                delta_uv1 = vec2(0.0, 1.0);
                delta_uv2 = vec2(1.0, 0.0);
            }

            // Create tangent and bitangent vectors
            let mut tangent: Vector3<f32> = Vector3::zero();
            let mut bitangent: Vector3<f32> = Vector3::zero();

            // Calculate tangent vector
            tangent.x = dir_correction * (edge2.x * delta_uv1.y - edge1.x * delta_uv2.y);
            tangent.y = dir_correction * (edge2.y * delta_uv1.y - edge1.y * delta_uv2.y);
            tangent.z = dir_correction * (edge2.z * delta_uv1.y - edge1.z * delta_uv2.y);
            
            // Calculate bitangent vector
            bitangent.x = dir_correction * ( - edge2.x * delta_uv1.x + edge1.x * delta_uv2.x);
            bitangent.y = dir_correction * ( - edge2.y * delta_uv1.x + edge1.y * delta_uv2.x);
            bitangent.z = dir_correction * ( - edge2.z * delta_uv1.x + edge1.z * delta_uv2.x);

            // Set tangent vector to all vertices of the triangle
            vertices[index1].tangent = tangent;
            vertices[index2].tangent = tangent;
            vertices[index3].tangent = tangent;            

            vertices[index1].bitangent = bitangent;
            vertices[index2].bitangent = bitangent;
            vertices[index3].bitangent = bitangent;
        }
    }

    pub fn reset_material(shader_program: &ShaderProgram) -> Result<(), GlError> {
        shader_program.set_bool("material.has_diffuse", false, true)?;
        shader_program.set_bool("material.has_specular", false, true)?;
        shader_program.set_bool("material.has_normal", false, true)?;
        shader_program.set_bool("material.has_displacement", false, true)?;

        Ok(())
    }

    pub fn set_texture(texture: &Texture, shader_program: &ShaderProgram, num: u32) -> Result<(), GlError> {
        texture.ready_texture(num);

        // TODO: make this a texture array ig? Ignores numbers for now (texture arrays are a thing!)
        let name = &texture.type_;
        shader_program.set_int(format!("material.{}", name).as_str(), num as i32, true)?;
        shader_program.set_bool(format!("material.has_{}", name).as_str(), true, true)?;

        Ok(())
    }

    pub fn set_material(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        Mesh::reset_material(shader_program)?;

        // TODO: Use a while loop here with an explicit iterator to avoid resizing i, benchmark and test if it's more optimized
        for (i, texture) in self.textures.iter().enumerate() {
            Mesh::set_texture(texture, shader_program, i as u32)?;
        }

        Ok(())
    }

    // TODO: supposedly inline functions are faster, draw calls should probably all be like this?
    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        self.set_material(shader_program)?;
        self.vao.draw_elements(self.ebo.len() as i32, self.tbo.len() as i32);

        unsafe {
            // Set back to defaults once configured
            gl::ActiveTexture(gl::TEXTURE0);
        }

        Ok(())
    }
}
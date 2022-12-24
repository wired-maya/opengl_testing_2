use std::rc::Rc;

use cgmath::{Vector3, Zero, Matrix4, vec2};
use memoffset::offset_of;
use super::{ShaderProgram, GlError, VertexArray, Buffer, Vertex, Texture};

// TODO: sort any meshes with alpha values and render them farthest to closest w/o depth buffer

pub struct Mesh {
    pub diffuse_textures: Vec<Rc<Texture>>,
    pub specular_textures: Vec<Rc<Texture>>,
    pub normal_textures: Vec<Rc<Texture>>,
    pub displacement_textures: Vec<Rc<Texture>>,
    pub vao: VertexArray,
    pub vbo: Buffer<Vertex>,
    pub ebo: Buffer<u32>,
    pub tbo: Buffer<Matrix4<f32>>
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, model_transforms: Vec<Matrix4<f32>>) -> Mesh {
        let mut mesh = Mesh {
            // TODO: Use data types with less overhead, like array instead of Vec<> (though do research if indexing is really slower)
            diffuse_textures: Vec::new(),
            specular_textures: Vec::new(),
            normal_textures: Vec::new(),
            displacement_textures: Vec::new(),
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

    fn set_textures(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        let mut i = 0;
        // TODO: use if statements to make ignore not found false, possibly remove ignore not found
        // TODO: Use a while loop here with an explicit iterator to avoid resizing i, benchmark and test if it's more optimized
        for texture in self.diffuse_textures.iter() {
            texture.ready_texture(i as u32);
            shader_program.set_int(format!("material.diffuse[{}]", i).as_str(), i as i32, true)?;
            i += 1;
        }

        shader_program.set_int("material.diffuseCount", self.diffuse_textures.len() as i32, true)?;

        for texture in self.specular_textures.iter() {
            texture.ready_texture(i as u32);
            shader_program.set_int(format!("material.specular[{}]", i).as_str(), i as i32, true)?;
            i += 1;
        }

        shader_program.set_int("material.specularCount", self.specular_textures.len() as i32, true)?;

        for texture in self.normal_textures.iter() {
            texture.ready_texture(i as u32);
            shader_program.set_int(format!("material.normal[{}]", i).as_str(), i as i32, true)?;
            i += 1;
        }

        shader_program.set_int("material.normalCount", self.normal_textures.len() as i32, true)?;

        for texture in self.displacement_textures.iter() {
            texture.ready_texture(i as u32);
            shader_program.set_int(format!("material.displacement[{}]", i).as_str(), i as i32, true)?;
            i += 1;
        }

        shader_program.set_int("material.displacementCount", self.displacement_textures.len() as i32, true)?;

        Ok(())
    }

    // TODO: supposedly inline functions are faster, draw calls should probably all be like this?
    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        self.set_textures(shader_program)?;
        self.vao.draw_elements(self.ebo.len() as i32, self.tbo.len() as i32);

        unsafe {
            // Set back to defaults once configured
            gl::ActiveTexture(gl::TEXTURE0);
        }

        Ok(())
    }
}
use crate::gl_safe::ShaderProgram;
use super::GlError;

// UBO can have multiple types of data, so it doesn't have a type
pub struct UniformBuffer {
    id: u32
}

impl UniformBuffer {
    pub fn new(shader_programs: &[&ShaderProgram], name: &str, buffer_size: u32) -> Result<UniformBuffer, GlError> {
        let mut uniform_buffer = UniformBuffer {
            id: 0
        };

        for (_, shader_program) in shader_programs.iter().enumerate() {
            shader_program.bind_to_ubo(name)?;
        }

        uniform_buffer.create_ubo(buffer_size);

        Ok(uniform_buffer)
    }

    pub fn create_ubo(&mut self, buffer_size: u32) {
        unsafe {
            gl::GenBuffers(1, &mut self.id);

            gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);
            gl::BufferData(gl::UNIFORM_BUFFER, buffer_size as isize, std::ptr::null(), gl::DYNAMIC_DRAW);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, self.id, 0, buffer_size as isize);
        }
    }

    pub fn write_data<T>(&self, data: *const gl::types::GLvoid, offset: u32) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);
            gl::BufferSubData(gl::UNIFORM_BUFFER, offset as isize, std::mem::size_of::<T>() as isize, data);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}
use crate::gl_safe::ShaderProgram;
use super::GlError;

// UBO can have multiple types of data, so it doesn't have a type
pub struct UniformBuffer {
    id: u32,
    name: String
}

impl UniformBuffer {
    pub fn new(shader_programs: Vec<&ShaderProgram>, name: &str, buffer_size: u32) -> Result<UniformBuffer, GlError> {
        let mut uniform_buffer = UniformBuffer {
            id: 0,
            name: String::from(name)
        };

        for shader_program in shader_programs.iter() {
            uniform_buffer.register_shader_program(shader_program)?;
        }

        uniform_buffer.create_ubo(buffer_size);

        Ok(uniform_buffer)
    }

    pub fn register_shader_program(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        shader_program.bind_to_ubo(self.name.as_str())
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

// TODO: delete when dropped
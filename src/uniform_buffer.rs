use std::ffi::CString;

use crate::shader_program::ShaderProgram;

pub struct UniformBuffer {
    name: String,
    ubo: u32
}

impl UniformBuffer {
    pub fn new(shader_programs: &[&ShaderProgram], name: &str, buffer_size: u32) -> UniformBuffer {
        let mut uniform_buffer = UniformBuffer {
            name: name.to_owned(),
            ubo: 0
        };

        unsafe {
            for (_, shader_program) in shader_programs.iter().enumerate() {
                uniform_buffer.bind_shaders(shader_program);
            }

            uniform_buffer.create_ubo(buffer_size);
        }

        uniform_buffer
    }

    unsafe fn bind_shaders(&mut self, shader_program: &ShaderProgram) {
        let cstr = CString::new(&self.name[..]).unwrap();
        let uniform_block_index = gl::GetUniformBlockIndex(shader_program.id, cstr.as_ptr());

        gl::UniformBlockBinding(shader_program.id, uniform_block_index, 0);
    }

    unsafe fn create_ubo(&mut self, buffer_size: u32) {
        gl::GenBuffers(1, &mut self.ubo);

        gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
        gl::BufferData(gl::UNIFORM_BUFFER, buffer_size as isize, std::ptr::null(), gl::DYNAMIC_DRAW);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

        gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, self.ubo, 0, buffer_size as isize);
    }

    pub unsafe fn write_data<T>(&self, data: *const gl::types::GLvoid, offset: u32) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
        gl::BufferSubData(gl::UNIFORM_BUFFER, offset as isize, std::mem::size_of::<T>() as isize, data);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }
}
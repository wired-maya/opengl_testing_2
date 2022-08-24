use std::ffi::{CString, CStr};
use std::fs::File;
use std::io::Read;
use std::ptr;

pub struct ShaderProgram {
    pub id: u32
}

impl ShaderProgram {
    pub fn new(vertex_path: &str, fragment_path: &str) -> ShaderProgram {
        let mut shader_program = ShaderProgram { id: 0 };
        let mut vert_shader_file = File::open(vertex_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", vertex_path));
        let mut frag_shader_file = File::open(fragment_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", fragment_path));

        let mut vert_shader_code = String::new();
        let mut frag_shader_code = String::new();

        vert_shader_file
            .read_to_string(&mut vert_shader_code)
            .expect("Failed to read vertex shader");
        frag_shader_file
            .read_to_string(&mut frag_shader_code)
            .expect("Failed to read fragment shader");

        let vert_shader_code = CString::new(vert_shader_code.as_bytes()).unwrap();
        let frag_shader_code = CString::new(frag_shader_code.as_bytes()).unwrap();

        unsafe {
            let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vert_shader, 1, &vert_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(vert_shader);
            ShaderProgram::check_compile_errors(vert_shader, "VERTEX");

            let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(frag_shader, 1, &frag_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(frag_shader);
            ShaderProgram::check_compile_errors(frag_shader, "FRAGMENT");

            let shader_program_id = gl::CreateProgram();
            gl::AttachShader(shader_program_id, vert_shader);
            gl::AttachShader(shader_program_id, frag_shader);
            gl::LinkProgram(shader_program_id);
            ShaderProgram::check_compile_errors(shader_program_id, "PROGRAM");
            gl::DeleteShader(vert_shader);
            gl::DeleteShader(frag_shader);

            shader_program.id = shader_program_id;
        }

        shader_program
    }

    unsafe fn check_compile_errors(id: u32, type_: &str) {
        let mut success = gl::FALSE as gl::types::GLint;

        if type_ == "PROGRAM" {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                let mut len: gl::types::GLint = 0;
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let error = {
                    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                    buffer.extend([b' '].iter().cycle().take(len as usize));
                    CString::from_vec_unchecked(buffer)
                };

                gl::GetProgramInfoLog(
                    id,
                    len,
                    ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
                println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", error.to_string_lossy().into_owned());
            }
        } else {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                let mut len: gl::types::GLint = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let error = {
                    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
                    buffer.extend([b' '].iter().cycle().take(len as usize));
                    CString::from_vec_unchecked(buffer)
                };

                gl::GetShaderInfoLog(
                    id,
                    len,
                    ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
                println!("ERROR::SHADER::{}::COMPILATION_FAILED\n{}", type_, error.to_string_lossy().into_owned());
            }
        }
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn _set_bool(&self, name: &CStr, value: bool) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as gl::types::GLint);
    }

    pub unsafe fn _set_int(&self, name: &CStr, value: i32) {
        gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as gl::types::GLint);
    }

    pub unsafe fn _set_float(&self, name: &CStr, value: f32) {
        gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value as gl::types::GLfloat);
    }
}
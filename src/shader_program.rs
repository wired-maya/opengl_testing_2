use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::ptr;

use cgmath::{Vector3, Array, Matrix4, Matrix};

pub struct ShaderProgram {
    pub id: u32
}

impl ShaderProgram {
    pub fn new(vertex_path: &str, fragment_path: &str, maybe_geometry_path: Option<&str>) -> ShaderProgram {
        let mut shader_program = ShaderProgram { id: 0 };

        unsafe {
            let vert_shader = ShaderProgram::compile_shader(vertex_path, "VERTEX");
            let frag_shader = ShaderProgram::compile_shader(fragment_path, "FRAGMENT");

            // Geometry shader is the only one that is optional
            let geom_shader = if let Some(geometry_path) = maybe_geometry_path {
                ShaderProgram::compile_shader(geometry_path, "GEOMETRY")
            } else {
                0
            };

            let shader_program_id = gl::CreateProgram();

            gl::AttachShader(shader_program_id, vert_shader);
            gl::AttachShader(shader_program_id, frag_shader);
            if geom_shader != 0 { gl::AttachShader(shader_program_id, geom_shader); }

            gl::LinkProgram(shader_program_id);
            ShaderProgram::check_compile_errors(shader_program_id, "PROGRAM");

            gl::DeleteShader(vert_shader);
            gl::DeleteShader(frag_shader);
            if geom_shader != 0 { gl::DeleteShader(geom_shader); }

            shader_program.id = shader_program_id;
        }

        shader_program
    }

    unsafe fn compile_shader(path: &str, type_: &str) -> u32 {
        let mut shader_file = File::open(path)
            .unwrap_or_else(|_| panic!("Failed to open {}", path));
        let mut shader_code = String::new();

        shader_file
            .read_to_string(&mut shader_code)
            .unwrap_or_else(|_| panic!("Failed to read {} shader", type_.to_lowercase()));

        let shader_code = CString::new(shader_code.as_bytes()).unwrap();
        let shader = gl::CreateShader(match type_ {
            "VERTEX" => gl::VERTEX_SHADER,
            "GEOMETRY" => gl::GEOMETRY_SHADER,
            "FRAGMENT" => gl::FRAGMENT_SHADER,
            _ => gl::VERTEX_SHADER // Default to vertex shader just in case
        });

        gl::ShaderSource(shader, 1, &shader_code.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        ShaderProgram::check_compile_errors(shader, type_);

        shader
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

    pub unsafe fn _set_bool(&self, name: &str, value: bool) {
        let cstr = CString::new(name).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(self.id, cstr.as_ptr()), value as gl::types::GLint);
    }

    pub unsafe fn set_int(&self, name: &str, value: i32) {
        let cstr = CString::new(name).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(self.id, cstr.as_ptr()), value as gl::types::GLint);
    }

    pub unsafe fn set_float(&self, name: &str, value: f32) {
        let cstr = CString::new(name).unwrap();
        gl::Uniform1f(gl::GetUniformLocation(self.id, cstr.as_ptr()), value as gl::types::GLfloat);
    }

    pub unsafe fn set_vector_3(&self, name: &str, value: &Vector3<f32>) {
        let cstr = CString::new(name).unwrap();
        gl::Uniform3fv(gl::GetUniformLocation(self.id, cstr.as_ptr()), 1, value.as_ptr());
    }

    pub unsafe fn set_vec3(&self, name: &str, x: f32, y: f32, z: f32) {
        let cstr = CString::new(name).unwrap();
        gl::Uniform3f(gl::GetUniformLocation(self.id, cstr.as_ptr()), x, y, z);
    }

    pub unsafe fn set_mat4(&self, name: &str, value: &Matrix4<f32>) {
        let cstr = CString::new(name).unwrap();
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, cstr.as_ptr()), 1, gl::FALSE, value.as_ptr());
    }
}
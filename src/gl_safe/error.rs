use std::{fmt::Display, error::Error, ffi::NulError, io};

use super::ShaderCompileType;

#[derive(Debug)]
pub enum GlError{
    CStringError(NulError),
    UniformNotFound(String, u32),
    ShaderCompileError(ShaderCompileType, u32, String),
    IoError(io::Error)
}

impl Display for GlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlError::CStringError(nul_error) => write!(f, "{}", nul_error),
            GlError::UniformNotFound(uniform, id) => 
                write!(f, "Uniform '{}' was not found in shader {}", uniform, id),
            GlError::ShaderCompileError(type_, id, error) =>
                write!(f, "Shader '{}' with ID {} failed to compile:\n{}", type_, id, error),
            GlError::IoError(io_error) => write!(f, "{}", io_error)
        }
    }
}

impl Error for GlError {}

impl From<NulError> for GlError {
    fn from(err: NulError) -> Self {
        GlError::CStringError(err)
    }
}

impl From<io::Error> for GlError {
    fn from(err: io::Error) -> Self {
        GlError::IoError(err)
    }
}
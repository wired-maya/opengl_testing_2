use std::{fmt::Display, error::Error, ffi::NulError, io};

use super::ShaderCompileType;

#[derive(Debug)]
pub enum GlError{
    CStringError(NulError),
    UniformNotFound(String, u32),
    ShaderCompileError(ShaderCompileType, u32, String),
    IoError(io::Error),
    ImageError(image::ImageError),
    ObjLoadError(tobj::LoadError),
    UniformInvalidIndex(String, u32)
}

impl Display for GlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlError::CStringError(nul_error) => write!(f, "{}", nul_error),
            GlError::UniformNotFound(uniform, id) => 
                write!(f, "Uniform '{}' was not found in shader {}", uniform, id),
            GlError::ShaderCompileError(type_, id, error) =>
                write!(f, "Shader '{}' with ID {} failed to compile:\n{}", type_, id, error),
            GlError::IoError(io_error) => write!(f, "{}", io_error),
            GlError::ImageError(img_error) => write!(f, "{}", img_error),
            GlError::ObjLoadError(obj_err) => write!(f, "{}", obj_err),
            GlError::UniformInvalidIndex(ub_name, id) => {
                write!(f, "Uniform block '{}' was not found in shader {}", ub_name, id)
            }
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

impl From<image::ImageError> for GlError {
    fn from(err: image::ImageError) -> Self {
        GlError::ImageError(err)
    }
}

impl From<tobj::LoadError> for GlError {
    fn from(err: tobj::LoadError) -> Self {
        GlError::ObjLoadError(err)
    }
}
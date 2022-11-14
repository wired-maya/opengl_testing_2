use std::cell::RefCell;

use super::GlError;
use image::DynamicImage::*;

pub struct Texture {
    pub id: RefCell<u32>, // TODO: Refcell until framebuffer is rewritten, so it can mutate id
    pub target: gl::types::GLenum, // TODO: also needs to become private after framebuffer rewrite
    pub type_: String,
    pub path: String
}

impl Texture {
    // Creates 2D texture applied to currently bound texture
    pub unsafe fn from_file_to_bound(path: &str, target: gl::types::GLenum) -> Result<(), GlError> {
        let img = image::io::Reader::open(path)?.decode()?;
        let data = img.as_bytes();

        // TODO: if there is an alpha, mark mesh as transparent
        let (internal_format, data_format) = match img {
            ImageLuma8(_) => (gl::RED, gl::RED),
            ImageLumaA8(_) => (gl::RG, gl::RG),
            ImageRgb8(_) => (gl::SRGB, gl::RGB),
            ImageRgba8(_) => (gl::SRGB_ALPHA, gl::RGBA),
            _ => (gl::SRGB, gl::RGB) // If nothing else, try default
        };

        unsafe {
            gl::TexImage2D(
                target,
                0,
                internal_format as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                data_format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const gl::types::GLvoid
            );
        }

        Ok(())
    }

    pub fn from_file_2d(path: &str, type_: &str) -> Result<Texture, GlError> {
        let mut texture = Texture {
            id: RefCell::new(0),
            target: gl::TEXTURE_2D,
            type_: type_.to_owned(),
            path: path.to_owned()
        };
    
        unsafe {
            gl::GenTextures(1, texture.id.get_mut());
            gl::BindTexture(texture.target, *texture.id.borrow());
            
            Texture::from_file_to_bound(path, texture.target)?;
            
            gl::GenerateMipmap(texture.target);
            gl::TexParameteri(texture.target, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        Ok(texture)
    }

    pub fn from_file_cubemap(faces: Vec<String>) -> Result<Texture, GlError> {
        let mut texture = Texture {
            id: RefCell::new(0),
            target: gl::TEXTURE_CUBE_MAP,
            type_: "diffuse".to_owned(),
            path: "".to_owned()
        };

        unsafe {
            gl::GenTextures(1, texture.id.get_mut());
            gl::BindTexture(texture.target, *texture.id.borrow());

            for (i, face) in faces.iter().enumerate() {
                Texture::from_file_to_bound(face, gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32)?;
            }

            gl::TexParameteri(texture.target, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(texture.target, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        }

        Ok(texture)
    }

    pub fn ready_texture(&self, num: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + num);
            gl::BindTexture(self.target, *self.id.borrow());
        }
    }

    // TODO: Temp until framebuffer is rewritten
    pub fn set_id(&self, id: u32) {
        *self.id.borrow_mut() = id;
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &*self.id.borrow());
        }
    }
}
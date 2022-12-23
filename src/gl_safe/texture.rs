use std::rc::Rc;
use super::{GlError, Framebuffer};
use image::DynamicImage::*;

pub struct Texture {
    id: u32,
    target: gl::types::GLenum,
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

    pub fn from_file_2d(path: &str) -> Result<Texture, GlError> {
        let mut texture = Texture {
            id: 0,
            target: gl::TEXTURE_2D,
            path: path.to_owned()
        };
    
        unsafe {
            gl::GenTextures(1, &mut texture.id);
            gl::BindTexture(texture.target, texture.id);
            
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
            id: 0,
            target: gl::TEXTURE_CUBE_MAP,
            path: "".to_owned()
        };

        unsafe {
            gl::GenTextures(1, &mut texture.id);
            gl::BindTexture(texture.target, texture.id);

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

    // Doesn't need GlError since this only generates gl callback errors
    // Assumes framebuffer is bound
    pub fn for_framebuffer(framebuffer: &mut Framebuffer) -> (u32, Rc<Texture>) {
        let mut texture = Texture {
            id: 0,
            path: "".into(),
            target: gl::TEXTURE_2D
        };

        // Get number of new texture
        let num: u32 = framebuffer.len() as u32;
        let (width, height) = framebuffer.get_size();

        unsafe {
            gl::GenTextures(1, &mut texture.id);
            gl::BindTexture(texture.target, texture.id);

            // Create empty texture
            gl::TexImage2D(
                texture.target,
                0,
                gl::RGBA16F as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );

            // Nearest just for simplicity
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Bind to framebuffer
            gl::BindTexture(texture.target, 0);
            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0 + num,
                texture.id,
                0
            );
        }

        (gl::COLOR_ATTACHMENT0 + num, Rc::new(texture))
    }

    pub fn ready_texture(&self, num: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + num);
            gl::BindTexture(self.target, self.id);
        }
    }

    // TODO: Make resize inline as well since it will play during animations of 3D scenes
    // Unsafe because you need to know what you are doing so that you can resize without a mutable borrow
    pub unsafe fn resize(&self, width: i32, height: i32) {
        gl::BindTexture(self.target, self.id);
        // Resizes texture on same ID
        gl::TexImage2D(
            self.target,
            0,
            gl::RGBA16F as i32,
            width,
            height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null()
        );
        gl::BindTexture(self.target, 0);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
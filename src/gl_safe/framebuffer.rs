use std::rc::Rc;

use cgmath::{Matrix4, vec3};

use super::{GlError, Texture, RenderBuffer, Mesh, create_quad, ShaderProgram};

pub struct Framebuffer {
    id: u32,
    textures: Vec<Rc<Texture>>,
    draw_buffers: Vec<gl::types::GLenum>,
    quad: Mesh,
    pub width: i32,
    pub height: i32,
    pub render_buffer: Option<RenderBuffer>
}

impl Framebuffer {
    pub fn new(
        width: i32,
        height: i32,
        tex_num: usize,
        has_rb: bool
    ) -> Result<Framebuffer, GlError> {
        // Create quad mesh for framebuffer
        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];
        let quad = create_quad(model_transforms);
        
        let mut framebuffer = Framebuffer {
            id: 0,
            textures: Vec::new(),
            draw_buffers: Vec::new(),
            quad,
            width,
            height,
            render_buffer: None,
        };

        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer.id);
        }
            
        // Set up renderbuffer, all these assume framebuffer is bound
        framebuffer.bind();
        framebuffer.gen_textures(tex_num);
        if has_rb {
            RenderBuffer::push_to_framebuffer(&mut framebuffer);
        }
        framebuffer.check_status()?;
        Framebuffer::unbind();

        Ok(framebuffer)
    }

    pub fn gen_textures(&mut self, n: usize) {
        unsafe {
            // TODO: replace this with array textures in shader program
            let available_types = vec![
                "diffuse",
                "specular",
                "normal",
                "displacement"
            ];

            for i in 0..n {
                let (attachment, texture) = Texture::for_framebuffer(self, available_types[i]);
                
                self.textures.push(texture);
                self.draw_buffers.push(attachment);
            }

            // Bind color attachments to the buffer
            gl::DrawBuffers(
                self.draw_buffers.len() as i32,
                self.draw_buffers.as_ptr()
            );
        }
    }

    pub fn check_status(&self) -> Result<(), GlError> {
        unsafe {
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE {
                Ok(())
            } else {
                Err(GlError::FramebufferNotComplete(self.id))
            }
        }
    }

    // framebuffer output -> self input
    // Does not clear to allow for multiple linking in a render pipeline,
    // AKA you have to do it
    pub fn link_to(&mut self, framebuffer: &Framebuffer) {
        for texture in framebuffer.textures.iter() {
            self.quad.textures.push(Rc::clone(&texture));
        }
    }

    pub fn link_push(&mut self, texture: Rc<Texture>) {
        self.quad.textures.push(texture);
    }

    pub fn unlink(&mut self) {
        self.quad.textures.clear();
    }

    // Get output texture at index
    pub fn get(&self, index: usize) -> Option<Rc<Texture>> {
        if let Some(texture) = self.textures.get(index) {
            Some(Rc::clone(texture))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        self.bind();

        shader_program.use_program();
        self.quad.draw(shader_program)?;

        Ok(())
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

// TODO: move to its own file
// TODO: see if you can make this all static
pub struct DefaultFramebuffer {
    pub quad: Mesh
}

impl DefaultFramebuffer {
    pub fn new() -> DefaultFramebuffer {
        // Create quad mesh for framebuffer
        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];
        let quad = create_quad(model_transforms);

        DefaultFramebuffer { quad }
    }

    pub fn bind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn draw(&self, shader_program: &ShaderProgram) -> Result<(), GlError> {
        DefaultFramebuffer::bind();

        shader_program.use_program();
        self.quad.draw(shader_program)
    }
}
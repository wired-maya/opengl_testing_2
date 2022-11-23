use super::Framebuffer;

pub struct RenderBuffer {
    id: u32
}

impl RenderBuffer {
    // Requires framebuffer to be bound
    pub fn push_to_framebuffer(framebuffer: &mut Framebuffer) {
        let mut renderbuffer = RenderBuffer {
            id: 0
        };

        unsafe {
            gl::GenRenderbuffers(1, &mut renderbuffer.id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer.id);
            
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                framebuffer.width,
                framebuffer.height
            );
            
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                renderbuffer.id
            );
        }

        framebuffer.render_buffer = Some(renderbuffer);
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.id);
        }
    }
}
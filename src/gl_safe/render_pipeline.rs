use super::{Framebuffer, DefaultFramebuffer, ShaderProgram, GlError};

// TODO: render pipeline trait? Implement multisampling
pub struct RenderPipeline {
    g_framebuffer: Framebuffer,
    framebuffer: Framebuffer,
    ping_framebuffer: Framebuffer,
    pong_framebuffer: Framebuffer,
    default_framebuffer: DefaultFramebuffer,
    ping_pong_hoz: bool,
    ping_pong_first_iter: bool,
    width: u32,
    height: u32
}

// TODO: Refactor this whole struct into a render pipeline and multiple framebuffer structs,
// TODO: with the main render pipeline struct handling resizing, sending data, hot recompiling
// TODO: shaders, etc, maybe make framebuffer struct a generic struct you can spin the rest
// TODO: off of.
// TODO: Render pipeline could also just draw to provided framebuffer instead of having it hard-coded to
// TODO: allow for more flexibility, as well as the default framebuffer/main rendering pipeline being
// TODO: where everything is drawn to. Could have render pipeline have its own final quad so default
// TODO: pipeline is static? Either way, needs to have functions to draw to selected quad or framebuffer
// TODO: to allow for the widget system to work with this. Might be able to have the renderpipeline trait
// TODO: have a required "draw" function that returns a ref to the final framebuffer in the chain,
// TODO: then have default implemented functions that can draw to x framebuffer/quad/etc. that can be
// TODO: overriden.
impl RenderPipeline {
    pub fn new(width: u32, height: u32) -> RenderPipeline {
        // Create g_buffer for deferred shading
        let g_framebuffer = Framebuffer::new(
            width as i32,
            height as i32,
            3,
            true
        ).unwrap();

        // Create framebuffer with second colour attachment for lighting calculations and bloom
        let mut framebuffer = Framebuffer::new(
            width as i32,
            height as i32,
            2,
            true
        ).unwrap();

        // Create two framebuffers to calculate bloom's blur
        let ping_framebuffer = Framebuffer::new(
            width as i32,
            height as i32,
            1,
            false
        ).unwrap();
        let pong_framebuffer = Framebuffer::new(
            width as i32,
            height as i32,
            1,
            false
        ).unwrap();

        // Create default framebuffer
        let default_framebuffer = DefaultFramebuffer::new();

        // Link all the framebuffers together
        framebuffer.link_to(&g_framebuffer);
        // The rest are linked in draw call

        RenderPipeline {
            g_framebuffer,
            framebuffer,
            ping_framebuffer,
            pong_framebuffer,
            default_framebuffer,
            ping_pong_hoz: true,
            ping_pong_first_iter: true,
            width,
            height
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            self.g_framebuffer.bind();
            
            // Colour buffer does not need to be cleared when skybox is active
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn draw(
        &mut self,
        fb_shader_program: &ShaderProgram,
        blur_shader_program: &ShaderProgram,
        lighting_pass_shader_program: &ShaderProgram
    ) -> Result<(), GlError> {
        unsafe { gl::Disable(gl::DEPTH_TEST) };

        self.framebuffer.draw(lighting_pass_shader_program)?;

        // ------------------
        // Draw gaussian blur
        // ------------------

        let amount = 10;
        self.ping_pong_hoz = true;
        self.ping_pong_first_iter = true;

        blur_shader_program.use_program();

        for _ in 0..amount {
            blur_shader_program.set_bool("horizontal", self.ping_pong_hoz, false)?;

            if self.ping_pong_first_iter {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_push(self.framebuffer.get(1).unwrap());
                self.ping_framebuffer.draw(blur_shader_program)?;
            } else if self.ping_pong_hoz {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_to(&self.pong_framebuffer);
                self.ping_framebuffer.draw(blur_shader_program)?;
            } else {
                self.pong_framebuffer.unlink();
                self.pong_framebuffer.link_to(&self.ping_framebuffer);
                self.pong_framebuffer.draw(blur_shader_program)?;
            }

            self.ping_pong_hoz = !self.ping_pong_hoz;
            self.ping_pong_first_iter = false;
        }

        // -----------------
        // Draw normal frame
        // -----------------

        // Set necessary textures
        self.default_framebuffer.quad.textures.clear();
        self.default_framebuffer.quad.textures.push(
            self.framebuffer.get(0).unwrap()
        );
        self.default_framebuffer.quad.textures.push(
            if self.ping_pong_hoz {
                self.ping_framebuffer.get(0).unwrap()
            } else {
                self.pong_framebuffer.get(0).unwrap()
            }
        );

        // Draw the quad mesh
        self.default_framebuffer.draw(fb_shader_program)?;

        unsafe { gl::Enable(gl::DEPTH_TEST) };

        Ok(())
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) {
        // // TODO: delete g_buffer objs, rbo, etc
        // // Delete old objects and textures
        // gl::DeleteFramebuffers(1, &mut self.fbo);
        // gl::DeleteFramebuffers(1, &mut self.intermediate_fbo);
        
        // for i in 0..self.tbos.len() {
        //     gl::DeleteTextures(1, &mut self.tbos[i]);
        //     gl::DeleteTextures(1, &mut self.ping_pong_tbos[i]);
        //     gl::DeleteTextures(1, &mut self.intermediate_tbos[i]);
        //     gl::DeleteFramebuffers(1, &mut self.ping_pong_fbos[i]);
        // }

        // // Change self width and height
        // self.width = width;
        // self.height = height;

        // // Run setupbuffer again
        // self.setup_buffer()
    }
}
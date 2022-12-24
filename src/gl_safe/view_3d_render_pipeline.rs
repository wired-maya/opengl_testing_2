use std::rc::Rc;
use super::{Framebuffer, ShaderProgram, GlError, RenderPipeline, Texture};

pub struct View3DRenderPipeline {
    deffered_fb: Framebuffer,
    lighting_pass_fb: Framebuffer,
    lighting_pass_shader_program: ShaderProgram,
    ping_framebuffer: Framebuffer,
    pong_framebuffer: Framebuffer,
    blur_shader_program: ShaderProgram,
    ping_pong_hoz: bool,
    ping_pong_first_iter: bool,
    width: i32,
    height: i32
}

impl View3DRenderPipeline {
    pub fn new(
        width: i32,
        height: i32,
        lighting_pass_shader_program: ShaderProgram,
        blur_shader_program: ShaderProgram,
    ) -> View3DRenderPipeline {
        // Create g_buffer for deferred shading
        let deffered_fb = Framebuffer::new(
            width,
            height,
            3,
            true
        ).unwrap();

        // Create framebuffer with second colour attachment for lighting calculations and bloom
        let mut lighting_pass_fb = Framebuffer::new(
            width,
            height,
            2,
            true
        ).unwrap();

        // Create two framebuffers to calculate bloom's blur
        let ping_framebuffer = Framebuffer::new(
            width,
            height,
            1,
            false
        ).unwrap();
        let pong_framebuffer = Framebuffer::new(
            width,
            height,
            1,
            false
        ).unwrap();

        // Link all the framebuffers together
        lighting_pass_fb.link_to_fb(&deffered_fb);
        // The rest are linked in draw call

        View3DRenderPipeline {
            deffered_fb,
            lighting_pass_fb,
            lighting_pass_shader_program,
            ping_framebuffer,
            pong_framebuffer,
            blur_shader_program,
            ping_pong_hoz: true,
            ping_pong_first_iter: true,
            width,
            height,
        }
    }
}

impl RenderPipeline for View3DRenderPipeline {
    fn bind(&self) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            self.deffered_fb.bind();
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    fn draw(&mut self) -> Result<(), GlError> {
        unsafe { gl::Disable(gl::DEPTH_TEST) };

        self.lighting_pass_fb.draw(&self.lighting_pass_shader_program)?;

        // ------------------
        // Draw gaussian blur
        // ------------------

        let amount = 10;
        self.ping_pong_hoz = true;
        self.ping_pong_first_iter = true;

        self.blur_shader_program.use_program();

        // TODO: Could there be a way to do this in one FB? Would cut down on links
        for _ in 0..amount {
            self.blur_shader_program.set_bool("horizontal", self.ping_pong_hoz)?;

            if self.ping_pong_first_iter {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_push(self.lighting_pass_fb.get(1).unwrap());
                self.ping_framebuffer.draw(&self.blur_shader_program)?;
            } else if self.ping_pong_hoz {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_to_fb(&self.pong_framebuffer);
                self.ping_framebuffer.draw(&self.blur_shader_program)?;
            } else {
                self.pong_framebuffer.unlink();
                self.pong_framebuffer.link_to_fb(&self.ping_framebuffer);
                self.pong_framebuffer.draw(&self.blur_shader_program)?;
            }

            self.ping_pong_hoz = !self.ping_pong_hoz;
            self.ping_pong_first_iter = false;
        }

        Ok(())
    }

    fn get_height(&self) -> (i32, i32) {
        return (self.width, self.height);
    }

    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError> {
        self.width = width;
        self.height = height;

        // Resize FBs
        self.deffered_fb.set_size(width, height);
        self.lighting_pass_fb.set_size(width, height);
        self.ping_framebuffer.set_size(width, height);
        self.pong_framebuffer.set_size(width, height);

        Ok(())
    }

    fn get_link(&self) -> Result<Vec<Rc<Texture>>, GlError> {
        return Ok(
            vec![self.lighting_pass_fb.get(0).unwrap(), self.ping_framebuffer.get(0).unwrap()]
        );
    }

    fn link_to(&mut self, output: Vec<Rc<Texture>>) -> Result<(), GlError> {
        for texture in output {
            self.deffered_fb.link_push(texture);
        }

        Ok(())
    }

    fn unlink(&mut self) {
        self.deffered_fb.unlink();
    }

    fn link_push(&mut self, texture: Rc<Texture>) -> Result<(), GlError> {
        self.deffered_fb.link_push(texture);

        Ok(())
    }
}
use std::rc::Rc;
use cgmath::{Matrix4, Matrix, Deg};
use super::{Framebuffer, ShaderProgram, GlError, RenderPipeline, Texture, Model, Camera, Skybox, UniformBuffer};

// TODO: Implement multisampling
// TODO: refactor so models, camera, skybox, that kinda stuff, is held in a scene struct instead
// TODO: Scene trait with a simple 3d scene struct, so you can have other scenes (like 2d scenes)
// TODO: See if qsort is fast enough that  to allow me to sort models based on distance from the camera every frame, enabling transparency
pub struct View3DRenderPipeline {
    g_framebuffer: Framebuffer,
    framebuffer: Framebuffer,
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
        let g_framebuffer = Framebuffer::new(
            width,
            height,
            3,
            true
        ).unwrap();

        // Create framebuffer with second colour attachment for lighting calculations and bloom
        let mut framebuffer = Framebuffer::new(
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
        framebuffer.link_to_fb(&g_framebuffer);
        // The rest are linked in draw call

        View3DRenderPipeline {
            g_framebuffer,
            framebuffer,
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
            self.g_framebuffer.bind();
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    fn draw(&mut self) -> Result<(), GlError> {
        unsafe { gl::Disable(gl::DEPTH_TEST) };

        self.framebuffer.draw(&self.lighting_pass_shader_program)?;

        // ------------------
        // Draw gaussian blur
        // ------------------

        let amount = 10;
        self.ping_pong_hoz = true;
        self.ping_pong_first_iter = true;

        self.blur_shader_program.use_program();

        // TODO: Could there be a way to do this in one FB? Would cut down on links
        for _ in 0..amount {
            self.blur_shader_program.set_bool("horizontal", self.ping_pong_hoz, false)?;

            if self.ping_pong_first_iter {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_push(self.framebuffer.get(1).unwrap());
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

    // TODO: Change all sizes to have a struct instead so the tuple order isn't ambiguous?
    fn get_height(&self) -> (i32, i32) {
        return (self.width, self.height);
    }

    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError> {
        self.width = width;
        self.height = height;

        // Resize FBs
        self.g_framebuffer.set_size(width, height);
        self.framebuffer.set_size(width, height);
        self.ping_framebuffer.set_size(width, height);
        self.pong_framebuffer.set_size(width, height);

        Ok(())
    }

    fn get_link(&self) -> Result<Vec<Rc<Texture>>, GlError> {
        return Ok(
            vec![self.framebuffer.get(0).unwrap(), self.ping_framebuffer.get(0).unwrap()]
        );
    }

    fn link_to(&mut self, output: Vec<Rc<Texture>>) -> Result<(), GlError> {
        for texture in output {
            self.g_framebuffer.link_push(texture);
        }

        Ok(())
    }

    fn unlink(&mut self) {
        self.g_framebuffer.unlink();
    }

    fn link_push(&mut self, texture: Rc<Texture>) -> Result<(), GlError> {
        self.g_framebuffer.link_push(texture);

        Ok(())
    }
}
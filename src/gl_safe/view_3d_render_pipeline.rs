use std::rc::Rc;

use cgmath::{EuclideanSpace, Matrix4, Matrix, Deg};

use super::{Framebuffer, DefaultFramebuffer, ShaderProgram, GlError, RenderPipeline, Texture, Model, Camera, Skybox, UniformBuffer};

// TODO: Implement multisampling
// TODO: refactor so models, camera, skybox, that kinda stuff, is held in a scene struct instead
// TODO: Scene trait with a simple 3d scene struct, so you can have other scenes (like 2d scenes)
// TODO: See if qsort is fast enough that  to allow me to sort models based on distance from the camera every frame, enabling transparency
pub struct View3DRenderPipeline<'a> {
    g_framebuffer: Framebuffer,
    shader_program: &'a ShaderProgram,
    framebuffer: Framebuffer,
    lighting_pass_shader_program: &'a ShaderProgram,
    ping_framebuffer: Framebuffer,
    pong_framebuffer: Framebuffer,
    blur_shader_program: &'a ShaderProgram,
    ping_pong_hoz: bool,
    ping_pong_first_iter: bool,
    width: i32,
    height: i32,
    models: Vec<&'a Model>,
    camera: &'a Camera,
    skybox: Skybox,
    skybox_shader_program: &'a ShaderProgram,
    uniform_buffer: &'a UniformBuffer
}

impl<'a> View3DRenderPipeline<'a> {
    pub fn new(
        width: i32,
        height: i32,
        shader_program: &'a ShaderProgram,
        lighting_pass_shader_program: &'a ShaderProgram,
        blur_shader_program: &'a ShaderProgram,
        models: Vec<&'a Model>,
        camera: &'a Camera,
        skybox: Skybox,
        skybox_shader_program: &'a ShaderProgram,
        uniform_buffer: &'a UniformBuffer
    ) -> View3DRenderPipeline<'a> {
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
        framebuffer.link_to(&g_framebuffer);
        // The rest are linked in draw call

        View3DRenderPipeline {
            g_framebuffer,
            shader_program,
            framebuffer,
            lighting_pass_shader_program,
            ping_framebuffer,
            pong_framebuffer,
            blur_shader_program,
            ping_pong_hoz: true,
            ping_pong_first_iter: true,
            width,
            height,
            models,
            camera,
            skybox,
            skybox_shader_program,
            uniform_buffer
        }
    }
}

impl<'a> RenderPipeline for View3DRenderPipeline<'a> {
    fn bind(&self) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            self.g_framebuffer.bind();
            
            // Colour buffer does not need to be cleared when skybox is active
            // gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            // gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    fn draw(&mut self) -> Result<Vec<Rc<Texture>>, GlError> {
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        // Update view transforms
        let view_transform = self.camera.get_view_matrix();

        self.uniform_buffer.write_data::<Matrix4<f32>>(
            view_transform.as_ptr() as *const gl::types::GLvoid,
            std::mem::size_of::<Matrix4<f32>>() as u32
        );

        // Draw all models
        self.bind();
        self.shader_program.use_program();
        // TODO: Only used in forward shading
        // self.shader_program.set_vector_3("viewPos", &self.camera.position.to_vec(), false)?;

        for model in &self.models {
            model.draw(self.shader_program)?;
        }

        // Drawn last so it only is drawn over unused pixels, improving performance
        self.skybox.draw(self.skybox_shader_program)?;

        unsafe { gl::Disable(gl::DEPTH_TEST) };

        self.framebuffer.draw(&self.lighting_pass_shader_program)?;

        // ------------------
        // Draw gaussian blur
        // ------------------

        let amount = 10;
        self.ping_pong_hoz = true;
        self.ping_pong_first_iter = true;

        self.blur_shader_program.use_program();

        for _ in 0..amount {
            self.blur_shader_program.set_bool("horizontal", self.ping_pong_hoz, false)?;

            if self.ping_pong_first_iter {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_push(self.framebuffer.get(1).unwrap());
                self.ping_framebuffer.draw(&self.blur_shader_program)?;
            } else if self.ping_pong_hoz {
                self.ping_framebuffer.unlink();
                self.ping_framebuffer.link_to(&self.pong_framebuffer);
                self.ping_framebuffer.draw(&self.blur_shader_program)?;
            } else {
                self.pong_framebuffer.unlink();
                self.pong_framebuffer.link_to(&self.ping_framebuffer);
                self.pong_framebuffer.draw(&self.blur_shader_program)?;
            }

            self.ping_pong_hoz = !self.ping_pong_hoz;
            self.ping_pong_first_iter = false;
        }

        Ok(vec![self.framebuffer.get(0).unwrap(), if self.ping_pong_hoz {
            self.ping_framebuffer.get(0).unwrap()
        } else {
            self.pong_framebuffer.get(0).unwrap()
        }])
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

        // Resize camera
        let proj_transform = cgmath::perspective(
            Deg(self.camera.zoom),
            self.width as f32 / self.height as f32,
            0.1,
            500.0
        );
        self.uniform_buffer.write_data::<Matrix4<f32>>(proj_transform.as_ptr() as *const gl::types::GLvoid, 0);

        Ok(())
    }
}
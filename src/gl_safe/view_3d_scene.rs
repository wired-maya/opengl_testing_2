use super::{Model, Skybox, ShaderProgram, Camera, RenderPipeline, GlError, Scene};

// TODO: add lights, need a light trait
// TODO: See if qsort is fast enough that  to allow me to sort models based on distance from the camera every frame, enabling transparency
pub struct View3DScene {
    pub models: Vec<Model>,
    pub model_shader_program: ShaderProgram,
    pub skybox: Skybox,
    pub skybox_shader_program: ShaderProgram,
    pub camera: Camera,
    pub render_pipeline: Box<dyn RenderPipeline>,
}

impl Scene for View3DScene {
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError> {
        self.render_pipeline.set_size(width, height)?;
        self.camera.width = width as f32;
        self.camera.height = height as f32;
        self.camera.send_proj()?;

        Ok(())
    }

    fn draw(&mut self) -> Result<(), GlError> {
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        self.camera.send_view()?;

        self.render_pipeline.bind();
        self.model_shader_program.use_program();

        for model in self.models.iter() {
            model.draw(&self.model_shader_program)?;
        }

        // Drawn last so it only is drawn over unused pixels, improving performance
        self.skybox.draw(&self.skybox_shader_program)?;

        self.render_pipeline.draw()?;

        Ok(())
    }
}
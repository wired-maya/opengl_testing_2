use cgmath::{Matrix, Matrix4};

use super::{Model, Skybox, ShaderProgram, UniformBuffer, Camera, RenderPipeline, GlError};

// TODO: add lights, need a light trait
pub struct Scene {
    pub models: Vec<Model>,
    pub model_shader_program: ShaderProgram,
    pub skybox: Skybox,
    pub skybox_shader_program: ShaderProgram,
    pub camera: Camera,
    pub render_pipeline: Box<dyn RenderPipeline>,
    pub uniform_buffer: UniformBuffer
}

impl Scene {
    pub fn draw(&mut self) -> Result<(), GlError> {
        unsafe { gl::Enable(gl::DEPTH_TEST) };

        // Update view transforms
        let view_transform = self.camera.get_view_matrix();

        self.uniform_buffer.write_data::<Matrix4<f32>>(
            view_transform.as_ptr() as *const gl::types::GLvoid,
            std::mem::size_of::<Matrix4<f32>>() as u32
        );

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
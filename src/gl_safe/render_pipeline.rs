use std::rc::Rc;

use super::{GlError, Framebuffer, Texture, Mesh, UniformBuffer};

pub trait RenderPipeline {
    fn bind(&self);
    // Root function to render all effects and return an array of textures to render
    // (the texture array removes the need for a recombination of textures)
    fn draw(&mut self) -> Result<(), GlError>;
    fn get_height(&self) -> (i32, i32);
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError>;

    // Basic functions for linking, need to be defined by implemented so other funcs work
    fn get_link(&self) -> Result<Vec<Rc<Texture>>, GlError>;
    fn link_to(&mut self, output: Vec<Rc<Texture>>) -> Result<(), GlError>;
    fn link_push(&mut self, texture: Rc<Texture>) -> Result<(), GlError>;
    fn unlink(&mut self);

    // These are here to simplify and expand usage of render pipeline

    // render_pipeline output -> self input
    fn link_to_rp(&mut self, render_pipeline: &dyn RenderPipeline) -> Result<(), GlError> {
        self.link_to(render_pipeline.get_link()?)
    }
    fn link_to_fb(&mut self, fb: &mut Framebuffer) -> Result<(), GlError> {
        self.link_to(fb.get_link())
    }
    fn link_to_mesh(&mut self, mesh: &mut Mesh) -> Result<(), GlError> {
        for texture in mesh.textures.iter() {
            self.link_push(Rc::clone(&texture))?;
        }

        Ok(())
    }
}
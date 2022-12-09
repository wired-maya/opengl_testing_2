use std::rc::Rc;

use super::{GlError, Framebuffer, Texture, Mesh};

pub trait RenderPipeline {

    fn bind(&self);
    // Root function to render all effects and return an array of textures to render
    // (the texture array removes the need for a recombination of textures)
    fn draw(&mut self) -> Result<Vec<Rc<Texture>>, GlError>;
    fn get_height(&self) -> (i32, i32);
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError>;
    // These are here to simplify and expand usage of render pipeline
    // TODO: add functions for linking to framebuffer, meshes, and other render pipelines to avoid
    // TODO: having to clear and link them every frame
    fn draw_to_fb(&mut self, fb: &mut Framebuffer) -> Result<(), GlError> {
        let textures = self.draw()?;

        for texture in textures {
            fb.link_push(texture);
        }

        Ok(())
    }
    fn draw_to_mesh(&mut self, mesh: &mut Mesh) -> Result<(), GlError> {
        let textures = self.draw()?;

        for texture in textures {
            mesh.textures.push(texture);
        }

        Ok(())
    }
}
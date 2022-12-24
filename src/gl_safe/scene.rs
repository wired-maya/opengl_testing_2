use super::GlError;

pub trait Scene {
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), GlError>;
    fn draw(&mut self) -> Result<(), GlError>;
}
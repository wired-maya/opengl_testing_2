struct Framebuffer {
    pub fbo: u32
}

impl Framebuffer {
    pub fn new() -> Framebuffer {
        let mut framebuffer = Framebuffer {
            fbo: 0
        };

        unsafe { framebuffer.setup_buffer() }

        framebuffer
    }

    unsafe fn setup_buffer(&mut self) {
        
    }
}
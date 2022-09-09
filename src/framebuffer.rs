use cgmath::{Vector3, Vector2};

use crate::{shader_program::{ShaderProgram}, mesh::{Texture, Mesh, Vertex}};

pub struct Framebuffer {
    fbo: u32,
    rbo: u32,
    mesh: Mesh,
    width: u32,
    height: u32
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Framebuffer {
        // Create quad that will display framebuffer
        let texture = Texture {
            id: 0,
            type_: "diffuse".to_owned(),
            path: "".to_owned()
        };

        // Flat panel definition
        let vertices = vec![
            Vertex {
                position: Vector3::new(-1.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 1.0)
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(1.0, 0.0)
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(1.0, 1.0)
            }
        ];

        let indices = vec![
            0, 1, 2,
            0, 2, 3
        ];

        let textures = vec![
            texture
        ];

        let mesh = Mesh::new(vertices, indices, textures);

        let mut framebuffer = Framebuffer {
            fbo: 0,
            rbo: 0,
            mesh,
            width,
            height
        };

        unsafe { framebuffer.setup_buffer() }

        framebuffer
    }

    unsafe fn setup_buffer(&mut self) {
        gl::GenFramebuffers(1, &mut self.fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

        // Generate texture image/colour buffer
        gl::GenTextures(1, &mut self.mesh.textures[0].id);
        gl::BindTexture(gl::TEXTURE_2D, self.mesh.textures[0].id);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            self.width as i32,
            self.height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            std::ptr::null()
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::BindTexture(gl::TEXTURE_2D, 0);

        // Attach to currently bound fbo
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            self.mesh.textures[0].id,
            0
        );

        // Create rbo to hold depth and stencil buffers
        gl::GenRenderbuffers(1, &mut self.rbo);

        gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            self.width as i32,
            self.height as i32
        );
        
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

        // Bind rbo to fbo
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            self.rbo
        );

        // Throw error if buffer is incomplete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            panic!();
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn bind_buffer(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
    }

    pub unsafe fn draw(&self, shader_program: &ShaderProgram) {
        // Bind to default buffer to draw this framebuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Clear necessary buffers
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Set neccesary values
        gl::Disable(gl::DEPTH_TEST);

        // Draw the quad mesh
        self.mesh.draw(shader_program);

        // Bind to new framebuffer so future draw calls draw to it
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

        // Set neccasary values
        gl::Enable(gl::DEPTH_TEST);
    }

    // pub unsafe fn resize(&mut self, width: u32, height: u32) {
    //     // TODO: delete old objects and texture
    //     // TODO: change self width and height
    //     // TODO: run setupbuffer again
    // }
}
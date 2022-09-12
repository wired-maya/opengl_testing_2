use cgmath::{Vector3, Vector2, Matrix4, vec3};

use crate::{shader_program::{ShaderProgram}, mesh::{Texture, Mesh, Vertex}};

pub struct Framebuffer {
    fbo: u32,
    intermediate_fbo: u32,
    ms_tbo: u32,
    ms_rbo: u32,
    mesh: Mesh,
    width: u32,
    height: u32,
    msaa: u32
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, msaa: u32) -> Framebuffer {
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

        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];

        let mesh = Mesh::new(vertices, indices, textures, model_transforms);

        let mut framebuffer = Framebuffer {
            fbo: 0,
            intermediate_fbo: 0,
            ms_tbo: 0,
            ms_rbo: 0,
            mesh,
            width,
            height,
            msaa
        };

        unsafe { framebuffer.setup_buffer() }

        framebuffer
    }

    unsafe fn setup_buffer(&mut self) {
        gl::GenFramebuffers(1, &mut self.fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

        // Create multisampled colour attachment texture and bind it
        gl::GenTextures(1, &mut self.ms_tbo);
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.ms_tbo);
        gl::TexImage2DMultisample(
            gl::TEXTURE_2D_MULTISAMPLE,
            self.msaa as i32,
            gl::RGB,
            self.width as i32,
            self.height as i32,
            gl::TRUE
        );
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D_MULTISAMPLE,
            self.ms_tbo,
            0
        );
        
        // Create an equally multisampled render buffer object for depth and stencil attachments
        gl::GenRenderbuffers(1, &mut self.ms_rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.ms_rbo);
        gl::RenderbufferStorageMultisample(
            gl::RENDERBUFFER,
            self.msaa as i32,
            gl::DEPTH24_STENCIL8,
            self.width as i32,
            self.height as i32
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            self.ms_rbo
        );

        // Throw error if buffer is incomplete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            panic!();
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Configure second framebuffer purely for post-processing
        gl::GenFramebuffers(1, &mut self.intermediate_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.intermediate_fbo);

        // Create colour attachment texture
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
        // We only need colour buffer
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            self.mesh.textures[0].id,
            0
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
        // Blit multisampled buffer to normal colourbuffer of intermediate FBO
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.intermediate_fbo);
        gl::BlitFramebuffer(
            0, 0, self.width as i32, self.height as i32,
            0, 0, self.width as i32, self.height as i32,
            gl::COLOR_BUFFER_BIT, gl::NEAREST
        );

        // Bind to default buffer to draw this framebuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Clear necessary buffers (Bright magenta colour to spot problems)
        gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Set neccesary values
        gl::Disable(gl::DEPTH_TEST);

        // Draw the quad mesh
        shader_program.use_program();
        self.mesh.draw(shader_program);

        // Set neccasary values
        gl::Enable(gl::DEPTH_TEST);
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) {
        // Delete old objects and textures
        gl::DeleteFramebuffers(1, &mut self.fbo);
        gl::DeleteFramebuffers(1, &mut self.intermediate_fbo);
        gl::DeleteTextures(1, &mut self.ms_tbo);
        gl::DeleteTextures(1, &mut self.mesh.textures[0].id);
        gl::DeleteRenderbuffers(1, &mut self.ms_rbo);

        // Change self width and height
        self.width = width;
        self.height = height;

        // Run setupbuffer again
        self.setup_buffer()
    }
}
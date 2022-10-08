use cgmath::{Vector3, Vector2, Matrix4, vec3};

use crate::{shader_program::{ShaderProgram}, mesh::{Texture, Mesh, Vertex}};

pub struct Framebuffer {
    g_fbo: u32,
    g_pos: u32,
    g_normal: u32,
    g_albedo_spec: u32,
    g_rbo: u32,
    fbo: u32,
    intermediate_tbos: [u32; 2],
    intermediate_fbo: u32,
    tbos: [u32; 2],
    ping_pong_fbos: [u32; 2],
    ping_pong_tbos: [u32; 2],
    ping_pong_hoz: bool,
    ping_pong_first_iter: bool,
    mesh: Mesh,
    width: u32,
    height: u32,
    msaa: u32
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, msaa: u32) -> Framebuffer {
        // Create quad that will display framebuffer
        // These are temporarily called arbitrary things so that there are 3 unique textures
        let frame_texture = Texture {
            id: 0,
            type_: "diffuse".into(),
            path: "".into()
        };
        let bloom_texture = Texture {
            id: 0,
            type_: "specular".into(),
            path: "".into()
        };
        let pos_texture = Texture {
            id: 0,
            type_: "normal".into(),
            path: "".into()
        };

        // Flat panel definition
        let vertices = vec![
            Vertex {
                position: Vector3::new(-1.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 1.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(-1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, -1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(1.0, 0.0),
                ..Vertex::default()
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(1.0, 1.0),
                ..Vertex::default()
            }
        ];

        let indices = vec![
            0, 1, 2,
            0, 2, 3
        ];

        let textures = vec![
            frame_texture, bloom_texture, pos_texture
        ];

        let model_transforms = vec![Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0))];

        let mesh = Mesh::new(vertices, indices, textures, model_transforms);

        let mut framebuffer = Framebuffer {
            // TODO: use defaults to avoid all these 0s
            g_fbo: 0,
            g_pos: 0,
            g_normal: 0,
            g_albedo_spec: 0,
            g_rbo: 0,
            fbo: 0,
            intermediate_tbos: [0, 0],
            intermediate_fbo: 0,
            tbos: [0, 0],
            ping_pong_fbos: [0, 0],
            ping_pong_tbos: [0, 0],
            ping_pong_hoz: true,
            ping_pong_first_iter: true,
            mesh,
            width,
            height,
            msaa,
        };

        unsafe { framebuffer.setup_buffer() }

        framebuffer
    }

    unsafe fn setup_buffer(&mut self) {
        // TODO: move these separate steps to their own sub structs

        // TODO: this is the part you would replace with deferred shading,
        // TODO: deffered shading is done in this step, while the rendering to 3 different
        // TODO: textures happens before this

        // TODO: somehow make g_buffer multisampled?

        // ------------------------------------
        // Create g_buffer for deferred shading
        // ------------------------------------

        gl::GenFramebuffers(1, &mut self.g_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.g_fbo);

        // Position color buffer
        gl::GenTextures(1, &mut self.g_pos);
        gl::BindTexture(gl::TEXTURE_2D, self.g_pos);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            self.width as i32,
            self.height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null()
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            self.g_pos,
            0
        );

        // Normal color buffer
        gl::GenTextures(1, &mut self.g_normal);
        gl::BindTexture(gl::TEXTURE_2D, self.g_normal);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            self.width as i32,
            self.height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null()
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT1,
            self.g_normal,
            0
        );

        // Color + specular color buffer
        gl::GenTextures(1, &mut self.g_albedo_spec);
        gl::BindTexture(gl::TEXTURE_2D, self.g_albedo_spec);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA16F as i32,
            self.width as i32,
            self.height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null()
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT2,
            self.g_albedo_spec,
            0
        );

        // Tell OpenGL which color attachments we'll use
        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
        gl::DrawBuffers(3, attachments.as_ptr());

        // Create an equally render buffer object for depth and stencil attachments
        gl::GenRenderbuffers(1, &mut self.g_rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.g_rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            self.width as i32,
            self.height as i32
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            self.g_rbo
        );

        // Throw error if buffer is incomplete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            panic!();
        }

        // ------------------------------------------------------------------------------------
        // Create framebuffer with second colour attachment for lighting calculations and bloom
        // ------------------------------------------------------------------------------------

        gl::GenFramebuffers(1, &mut self.fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

        // Create multisampled colour attachment texture and bind it
        for i in 0..self.tbos.len() {
            // TODO: figure out if you can do this in one line
            gl::GenTextures(1, &mut self.tbos[i]);
            gl::BindTexture(gl::TEXTURE_2D, self.tbos[i]);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0 + i as u32,
                gl::TEXTURE_2D,
                self.tbos[i],
                0
            );
        }

        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
        gl::DrawBuffers(2, attachments.as_ptr());

        // Throw error if buffer is incomplete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            panic!();
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // -------------------------------------------------
        // Create two framebuffers to calculate bloom's blur
        // -------------------------------------------------

        gl::GenFramebuffers(2, &mut self.ping_pong_fbos[0]);
        gl::GenTextures(2, &mut self.ping_pong_tbos[0]);
        for i in 0..self.ping_pong_fbos.len() {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.ping_pong_fbos[i]);
            gl::BindTexture(gl::TEXTURE_2D, self.ping_pong_tbos[i]);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                self.ping_pong_tbos[i],
                0
            );

            // Throw error if buffer is incomplete
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
                panic!();
            }
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // -------------------------------------------------------
        // Configure second framebuffer purely for post-processing
        // -------------------------------------------------------

        gl::GenFramebuffers(1, &mut self.intermediate_fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.intermediate_fbo);

        for i in 0..self.intermediate_tbos.len() {
            // TODO: figure out if you can do this in one line
            gl::GenTextures(1, &mut self.intermediate_tbos[i]);
            gl::BindTexture(gl::TEXTURE_2D, self.intermediate_tbos[i]);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null()
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::FramebufferTexture(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0 + i as u32,
                self.intermediate_tbos[i],
                0
            );
        }

        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
        gl::DrawBuffers(2, attachments.as_ptr());

        // Throw error if buffer is incomplete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            panic!();
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn bind_buffer(&self) {
        gl::Viewport(0, 0, self.width as i32, self.height as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.g_fbo); // TODO: bind to deffered buffer
        
        // Colour buffer does not need to be cleared when skybox is active
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub unsafe fn draw(
        &mut self,
        fb_shader_program: &ShaderProgram,
        blur_shader_program: &ShaderProgram,
        lighting_pass_shader_program: &ShaderProgram
    ) {
        // TODO: handle drawing from deffered buffer to this buffer that handles combining them

        gl::Disable(gl::DEPTH_TEST);

        // ------------------------------------
        // Draw quad to handle deffered shading
        // ------------------------------------

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);

        self.mesh.textures[0].id = self.g_pos;
        self.mesh.textures[1].id = self.g_normal;
        self.mesh.textures[2].id = self.g_albedo_spec;

        lighting_pass_shader_program.use_program();
        self.mesh.draw(lighting_pass_shader_program);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        
        // ---------------------------------------------------------------------
        // Blit multisampled buffers to normal colourbuffers of intermediate FBO
        // ---------------------------------------------------------------------

        // gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        // for i in 0..self.tbos.len() {
        //     gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
        //     gl::ReadBuffer(gl::COLOR_ATTACHMENT0 + i as u32);
        //     gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.intermediate_fbo);
        //     gl::DrawBuffer(gl::COLOR_ATTACHMENT0 + i as u32);
        //     gl::BlitFramebuffer(
        //         0, 0, self.width as i32, self.height as i32,
        //         0, 0, self.width as i32, self.height as i32,
        //         gl::COLOR_BUFFER_BIT, gl::NEAREST
        //     );
        //     // TODO: check for blit error
        // }

        // ------------------
        // Draw gaussian blur
        // ------------------

        let amount = 10;
        self.ping_pong_hoz = true;
        self.ping_pong_first_iter = true;

        blur_shader_program.use_program();

        for _ in 0..amount {
            gl::BindFramebuffer(
                gl::FRAMEBUFFER,
                self.ping_pong_fbos[if self.ping_pong_hoz {1} else {0}]
            );
            blur_shader_program.set_bool("horizontal", self.ping_pong_hoz);

            self.mesh.textures[0].id = if self.ping_pong_first_iter {
                self.tbos[1]
            }
            else {
                self.ping_pong_tbos[if self.ping_pong_hoz {0} else {1}]
            };

            self.mesh.draw(blur_shader_program);

            self.ping_pong_hoz = !self.ping_pong_hoz;
            self.ping_pong_first_iter = false;
        
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        // -----------------
        // Draw normal frame
        // -----------------

        // Bind to default buffer to draw this framebuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // Clear necessary buffers (Bright magenta colour to spot problems)
        gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Set necessary textures
        self.mesh.textures[0].id = self.tbos[0];
        self.mesh.textures[1].id = self.ping_pong_tbos[if self.ping_pong_hoz {0} else {1}];

        // Draw the quad mesh
        fb_shader_program.use_program();
        self.mesh.draw(fb_shader_program);

        gl::Enable(gl::DEPTH_TEST);
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) {
        // TODO: delete g_buffer objs
        // Delete old objects and textures
        gl::DeleteFramebuffers(1, &mut self.fbo);
        gl::DeleteFramebuffers(1, &mut self.intermediate_fbo);
        
        for i in 0..self.tbos.len() {
            gl::DeleteTextures(1, &mut self.tbos[i]);
            gl::DeleteTextures(1, &mut self.ping_pong_tbos[i]);
            gl::DeleteTextures(1, &mut self.intermediate_tbos[i]);
            gl::DeleteFramebuffers(1, &mut self.ping_pong_fbos[i]);
        }

        // Change self width and height
        self.width = width;
        self.height = height;

        // Run setupbuffer again
        self.setup_buffer()
    }
}
use cgmath::{Vector3, Matrix4, vec3, point3, Deg};

use crate::shader_program::ShaderProgram;

pub struct DirLight {
    pub direction: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,

    depth_fbo: u32,
    depth_map: u32,
    shadow_res: u32
}

impl DirLight {
    pub fn new(
        direction: Vector3<f32>,
        ambient: Vector3<f32>,
        diffuse: Vector3<f32>,
        specular: Vector3<f32>,
        shadow_res: u32
    ) -> DirLight {
        let mut dir_light = DirLight {
            direction,
            ambient,
            diffuse,
            specular,
            depth_fbo: 0,
            depth_map: 0,
            shadow_res
        };

        unsafe {
            dir_light.gen_depth_map();
        }

        dir_light
    }

    pub unsafe fn send_lighting_data(&self, shader_program: &ShaderProgram) {
        shader_program.use_program();
        
        shader_program.set_vector_3("dirLight.direction", &self.direction);
        
        shader_program.set_vector_3("dirLight.ambient", &self.ambient);
        shader_program.set_vector_3("dirLight.diffuse", &self.diffuse);
        shader_program.set_vector_3("dirLight.specular", &self.specular);
    }

    unsafe fn gen_depth_map(&mut self) {
        gl::GenFramebuffers(1, &mut self.depth_fbo);
        gl::GenTextures(1, &mut self.depth_map);

        gl::BindTexture(gl::TEXTURE_2D, self.depth_map);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::DEPTH_COMPONENT32F as i32,
            self.shadow_res as i32,
            self.shadow_res as i32,
            0,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            std::ptr::null()
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
        let border_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border_color.as_ptr());

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::FramebufferTexture(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            self.depth_map,
            0
        );

        // Tell OpenGL that we aren't drawing anything with this buffer
        gl::DrawBuffer(gl::NONE);
        gl::ReadBuffer(gl::NONE);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn bind_buffer(&self) {
        gl::Viewport(0, 0, self.shadow_res as i32, self.shadow_res as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }

    pub unsafe fn bind_shadow_map(&self, shader_program: &ShaderProgram) {
        // Texture is 31 to ensure this doesn't interfere with meshes unless they implement
        // an absurd amount of textures
        let shadow_map: i32 = 31;

        gl::ActiveTexture(gl::TEXTURE0 + shadow_map as u32);

        shader_program.set_int("shadowMap", shadow_map);

        gl::BindTexture(gl::TEXTURE_2D, self.depth_map);
    }

    pub unsafe fn configure_shader_and_matrices(&self, shader_program: &ShaderProgram) {
        // let (near_plane, far_plane) = (1.0, 7.5);
        let (near_plane, far_plane) = (0.1, 200.0);
        let light_projection: Matrix4<f32> = cgmath::ortho(
            -10.0,
            10.0,
            -10.0,
            10.0,
            near_plane,
            far_plane
        );

        let light_view = Matrix4::<f32>::look_to_rh(
            point3(self.direction.x, self.direction.y, self.direction.z),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0)
        );

        let light_space_matrix = light_projection * light_view;

        // Send to shader
        shader_program.use_program();
        shader_program.set_mat4("lightSpaceMatrix", &light_space_matrix);

        // TODO: only run this when lighting position changes
    }
}

pub struct PointLight {
    pub position: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,

    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,

    // TODO: temp, handle this differently in actual engine, probably
    pub array_position: u8,

    depth_fbo: u32,
    depth_map: u32,
    shadow_res: u32
}

impl PointLight {
    pub fn new(
        position: Vector3<f32>,
        ambient: Vector3<f32>,
        diffuse: Vector3<f32>,
        specular: Vector3<f32>,
        constant: f32,
        linear: f32,
        quadratic: f32,
        array_position: u8,
        shadow_res: u32
    ) -> PointLight {
        let mut point_light = PointLight {
            position,
            ambient,
            diffuse,
            specular,
            constant,
            linear,
            quadratic,
            array_position,
            depth_fbo: 0,
            depth_map: 0,
            shadow_res
        };

        unsafe { point_light.gen_depth_map(); }

        point_light
    }

    pub unsafe fn send_lighting_data(&self, shader_program: &ShaderProgram) {
        shader_program.use_program();

        shader_program.set_vector_3(format!("pointLights[{}].position", self.array_position).as_str(), &self.position);

        shader_program.set_vector_3(format!("pointLights[{}].ambient", self.array_position).as_str(), &self.ambient);
        shader_program.set_vector_3(format!("pointLights[{}].diffuse", self.array_position).as_str(), &self.diffuse);
        shader_program.set_vector_3(format!("pointLights[{}].specular", self.array_position).as_str(), &self.specular);

        shader_program.set_float(format!("pointLights[{}].constant", self.array_position).as_str(), self.constant);
        shader_program.set_float(format!("pointLights[{}].linear", self.array_position).as_str(), self.linear);
        shader_program.set_float(format!("pointLights[{}].quadratic", self.array_position).as_str(), self.quadratic);
    }

    pub unsafe fn gen_depth_map(&mut self) {
        gl::GenFramebuffers(1, &mut self.depth_fbo);
        gl::GenTextures(1, &mut self.depth_map);

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.depth_map);
        for i in 0..6 {
            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i,
                0,
                gl::DEPTH_COMPONENT32F as i32,
                self.shadow_res as i32,
                self.shadow_res as i32,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null()
            );
        }

        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::FramebufferTexture(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            self.depth_map,
            0
        );

        // Tell OpenGL that we aren't drawing anything with this buffer
        gl::DrawBuffer(gl::NONE);
        gl::ReadBuffer(gl::NONE);

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    pub unsafe fn bind_buffer(&self) {
        gl::Viewport(0, 0, self.shadow_res as i32, self.shadow_res as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_fbo);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }

    pub unsafe fn bind_shadow_map(&self, shader_program: &ShaderProgram) {
        // Texture is 31 to ensure this doesn't interfere with meshes unless they implement
        // an absurd amount of textures
        let shadow_map: i32 = 30;

        gl::ActiveTexture(gl::TEXTURE0 + shadow_map as u32);

        shader_program.set_int("shadowCubeMap", shadow_map);

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.depth_map);
    }

    pub unsafe fn configure_shader_and_matrices(&self, shader_program: &ShaderProgram) {
        let (aspect, near_plane, far_plane) = (1.0, 1.0, 600.0);

        let light_projection: Matrix4<f32> = cgmath::perspective(
            Deg(90.0),
            aspect,
            near_plane,
            far_plane,
        );

        let light_transforms: Vec<Matrix4<f32>> = vec![
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0)),
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(-1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0)),
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)),
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, -1.0)),
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(0.0, 0.0, 1.0), vec3(0.0, -1.0, 0.0)),
            light_projection * Matrix4::look_to_rh(point3(self.position.x, self.position.y, self.position.z), self.position + vec3(0.0, 0.0, -1.0), vec3(0.0, -1.0, 0.0)),
        ];

        // Send to shader
        shader_program.use_program();
        for (i, light_transform) in light_transforms.iter().enumerate() {
            shader_program.set_mat4(format!("shadowMatrices[{}]", i).as_str(), light_transform);
        }
        shader_program.set_float("far_plane", far_plane);
        shader_program.set_vector_3("lightPos", &self.position);

        // TODO: only run this when lighting position changes
    }
}

pub struct SpotLight {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,

    pub cutoff: f32,
    pub outer_cutoff: f32,

    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
}

impl SpotLight {
    pub unsafe fn send_data(&self, shader_program: &ShaderProgram) {
        shader_program.use_program();
        
        shader_program.set_vector_3("spotLight.position", &self.position);
        shader_program.set_vector_3("spotLight.direction", &self.direction);

        shader_program.set_float("spotLight.cutOff", self.cutoff);
        shader_program.set_float("spotLight.outerCutOff", self.outer_cutoff);

        shader_program.set_float("spotLight.constant", self.constant);
        shader_program.set_float("spotLight.linear", self.linear);
        shader_program.set_float("spotLight.quadratic", self.quadratic);

        shader_program.set_vector_3("spotLight.ambient", &self.ambient);
        shader_program.set_vector_3("spotLight.diffuse", &self.diffuse);
        shader_program.set_vector_3("spotLight.specular", &self.specular);
    }
}
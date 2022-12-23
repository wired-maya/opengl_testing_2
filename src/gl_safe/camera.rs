use cgmath::{Vector3, Point3, vec3, Zero, Matrix4, InnerSpace, Deg, Matrix};

use super::{UniformBuffer, GlError, ShaderProgram};

#[derive(PartialEq, Clone, Copy)]
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    RIGHT,
    LEFT,
    UP,
    DOWN
}

pub enum CameraProjection {
    PERSPECTIVE,
    ORTHO
}

pub struct Camera {
    pub position: Point3<f32>,
    pub uniform_buffer: Option<UniformBuffer>,
    // Direction vectors
    pub front: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub world_up: Vector3<f32>,
    // Euler angles
    pub yaw: f32,
    pub pitch: f32,
    // Camera options
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
    pub width: f32,
    pub height: f32,
    // Perspective options
    pub near: f32,
    pub far: f32,
    pub projection: CameraProjection
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            uniform_buffer: None,
            front: Vector3 { x: 0.0, y: 0.0, z: -1.0 },
            up: Vector3::zero(),
            right: Vector3::zero(),
            world_up: Vector3::unit_y(),
            yaw: -90.0,
            pitch: 0.0,
            movement_speed: 15.0,
            mouse_sensitivity: 0.1,
            fov: 45.0,
            width: 1920.0,
            height: 1080.0,
            near: 0.1,
            far: 500.0,
            projection: CameraProjection::PERSPECTIVE,
        }
    }
}

impl Camera {
    pub fn new(width: i32, height: i32, fov: f32, shader_programs: Vec<&ShaderProgram>) -> Result<Camera, GlError> {
        let mut camera = Camera {
            width: width as f32,
            height: height as f32,
            fov,
            ..Default::default()
        };
        let uniform_buffer = UniformBuffer::new(
            shader_programs,
            "CameraMatrices",
            2 * std::mem::size_of::<Matrix4<f32>>() as u32
        )?;

        camera.uniform_buffer = Some(uniform_buffer);
        camera.update_camera_vectors();
        camera.send_proj()?;

        Ok(camera)
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::<f32>::look_to_rh(self.position, self.front, self.up)
    }

    pub fn get_proj_matrix(&self) -> Matrix4<f32> {
        match self.projection {
            CameraProjection::PERSPECTIVE => cgmath::perspective(
                Deg(self.fov),
                self.width / self.height,
                self.near,
                self.far
            ),
            CameraProjection::ORTHO => cgmath::ortho(
                0.0,
                self.width,
                0.0,
                self.height,
                self.near,
                self.far
            ),
        }
    }

    pub fn process_movement(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;

        self.position += match direction {
            CameraMovement::FORWARD => self.front * velocity,
            CameraMovement::BACKWARD => -(self.front * velocity),
            CameraMovement::RIGHT => self.right * velocity,
            CameraMovement::LEFT => -(self.right * velocity),
            CameraMovement::UP => self.up * velocity,
            CameraMovement::DOWN => -(self.up * velocity),
        }
    }

    pub fn process_mouse_movement(&mut self, mut x_offset: f32, mut y_offset: f32, constrain_pitch: bool) {
        x_offset *= self.mouse_sensitivity;
        y_offset *= self.mouse_sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    pub fn update_camera_vectors(&mut self) {
        let front = vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
        );

        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn send_view(&self) -> Result<(), GlError> {
        let view_transform = self.get_view_matrix();

        if let Some(ubo) = &self.uniform_buffer {
            ubo.write_data::<Matrix4<f32>>(
                view_transform.as_ptr() as *const gl::types::GLvoid,
                std::mem::size_of::<Matrix4<f32>>() as u32
            );

            return Ok(());
        }

        Err(GlError::UniformBufferMissing)
    }

    pub fn send_proj(&self) -> Result<(), GlError> {
        let proj_transform = self.get_proj_matrix();

        if let Some(ubo) = &self.uniform_buffer {
            ubo.write_data::<Matrix4<f32>>(
                proj_transform.as_ptr() as *const gl::types::GLvoid,
                0
            );

            return Ok(());
        }

        Err(GlError::UniformBufferMissing)
    }
}
use cgmath::{Vector3, Point3, vec3, Zero, Matrix4, InnerSpace};

#[derive(PartialEq, Clone, Copy)]
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT
}

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVITY: f32 = 0.1;
const FOV: f32 = 45.0;

pub struct Camera {
    pub position: Point3<f32>,
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
    pub zoom: f32
}

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            position: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            front: vec3(0.0, 0.0, -1.0),
            up: Vector3::zero(),
            right: Vector3::zero(),
            world_up: Vector3::unit_y(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: FOV
        };
        camera.update_camera_vectors();
        camera
    }
}

impl Camera {
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::<f32>::look_to_rh(self.position, self.front, self.up)
    }

    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        if direction == CameraMovement::FORWARD {
            self.position += self.front * velocity;
        }
        if direction == CameraMovement::BACKWARD {
            self.position += -(self.front * velocity);
        }
        if direction == CameraMovement::LEFT {
            self.position += -(self.right * velocity);
        }
        if direction == CameraMovement::RIGHT {
            self.position += self.right * velocity;
        }

        self.position.y = 10.0; // Temporary
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

    pub fn process_mouse_scroll(&mut self, y_offset: f32) {
        if self.zoom >= 1.0 && self.zoom <= 45.0 {
            self.zoom -= y_offset;
        }
        if self.zoom <= 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom >= 45.0 {
            self.zoom = 45.0;
        }
    }

    fn update_camera_vectors(&mut self) {
        let front = vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
        );

        self.front = front.normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
use cgmath::Vector3;

use crate::shader_program::ShaderProgram;

pub struct DirLight {
    direction: Vector3<f32>,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>
}

impl DirLight {
    pub unsafe fn send_data(&self, shader_program: &ShaderProgram) {
        shader_program.set_vector_3("dirLight.direction", &self.direction);
        shader_program.set_vector_3("dirLight.ambient", &self.ambient);
        shader_program.set_vector_3("dirLight.diffuse", &self.diffuse);
        shader_program.set_vector_3("dirLight.specular", &self.specular);
    }
}

pub struct PointLight {
    position: Vector3<f32>,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,

    constant: f32,
    linear: f32,
    quadratic: f32,
}

pub struct SpotLight {
    position: Vector3<f32>,
    direction: Vector3<f32>,
    cutoff: f32,
    outer_cutoff: f32,

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
}

pub enum LightType {
    DIRECTIONAL(DirLight),
    POINT(PointLight),
    SPOT(SpotLight)
}

pub struct Light {
    type_: LightType
}
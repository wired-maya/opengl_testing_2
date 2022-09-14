use cgmath::Vector3;

use crate::shader_program::ShaderProgram;

pub struct DirLight {
    pub direction: Vector3<f32>,

    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>
}

impl DirLight {
    pub unsafe fn send_data(&self, shader_program: &ShaderProgram) {
        shader_program.use_program();
        
        shader_program.set_vector_3("dirLight.direction", &self.direction);
        
        shader_program.set_vector_3("dirLight.ambient", &self.ambient);
        shader_program.set_vector_3("dirLight.diffuse", &self.diffuse);
        shader_program.set_vector_3("dirLight.specular", &self.specular);
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
}

impl PointLight {
    pub unsafe fn send_data(&self, shader_program: &ShaderProgram) {
        shader_program.use_program();

        shader_program.set_vector_3(format!("pointLights[{}].position", self.array_position).as_str(), &self.position);

        shader_program.set_vector_3(format!("pointLights[{}].ambient", self.array_position).as_str(), &self.ambient);
        shader_program.set_vector_3(format!("pointLights[{}].diffuse", self.array_position).as_str(), &self.diffuse);
        shader_program.set_vector_3(format!("pointLights[{}].specular", self.array_position).as_str(), &self.specular);

        shader_program.set_float(format!("pointLights[{}].constant", self.array_position).as_str(), self.constant);
        shader_program.set_float(format!("pointLights[{}].linear", self.array_position).as_str(), self.linear);
        shader_program.set_float(format!("pointLights[{}].quadratic", self.array_position).as_str(), self.quadratic);
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
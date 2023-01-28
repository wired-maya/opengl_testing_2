#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 5) in mat4 model;
layout (std140) uniform CameraMatrices {
    mat4 projection;
    mat4 view;
};

flat out int instanceID;

void main() {
    instanceID = gl_InstanceID;

    vec4 vertex = vec4(aPos, 1.0);
    vec4 vertexView = view * model * vertex;
    gl_Position = projection * vertexView;
}
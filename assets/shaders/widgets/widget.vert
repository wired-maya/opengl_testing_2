#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 2) in vec2 aTexCoords;
layout (location = 5) in mat4 model;

out vec2 TexCoords;

void main() {
    vec4 vertex = vec4(aPos, 1.0);
    gl_Position = model * vertex;
    TexCoords = aTexCoords;
}
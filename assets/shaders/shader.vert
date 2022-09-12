#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;
layout (location = 3) in mat4 model;
layout (std140) uniform Matrices {
    mat4 projection;
    mat4 view;
};

out VS_OUT {
    vec2 texCoord;
    vec3 Normal;
    vec3 fragPos;
} vs_out;

void main() {
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    
    mat3 normalMatrix = mat3(transpose(inverse(view * model)));

    vs_out.texCoord = aTexCoord;
    vs_out.Normal = normalize(mat3(transpose(inverse(model))) * aNormal);
    // vs_out.Normal = normalize(vec3(vec4(normalMatrix * aNormal, 0.0)));
    vs_out.fragPos = vec3(model * vec4(aPos, 1.0));
}
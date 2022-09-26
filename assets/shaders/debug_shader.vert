#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoord;
layout (location = 3) in vec3 aTangent;
layout (location = 4) in vec3 aBitangent;
layout (location = 5) in mat4 model;
layout (std140) uniform Matrices {
    mat4 projection;
    mat4 view;
};

out VS_OUT {
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
} vs_out;

void main() {
    gl_Position = view * model * vec4(aPos, 1.0);
    mat3 normalMatrix = mat3(transpose(inverse(view * model)));
    vs_out.normal = normalize(vec3(vec4(normalMatrix * aNormal, 0.0)));
    vs_out.tangent = normalize(vec3(vec4(normalMatrix * aTangent, 0.0)));
    vs_out.bitangent = normalize(vec3(vec4(normalMatrix * aBitangent, 0.0)));
}
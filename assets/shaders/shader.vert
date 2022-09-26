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
    vec2 texCoord;
    vec4 FragPosLightSpace;
    vec3 TangentPointLightPosition;
    vec3 TangentDirLightDir;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    mat3 normalMatrix;
} vs_out;

uniform mat4 lightSpaceMatrix;
uniform vec3 viewPos;

uniform vec3 pointLightPosition;
uniform vec3 dirLightDir;

void main() {  
    vs_out.texCoord = aTexCoord;
    vec3 fragPos = vec3(model * vec4(aPos, 1.0));

    vs_out.FragPosLightSpace = lightSpaceMatrix * vec4(fragPos, 1.0);

    // Calculate matrix to transform normals based on model matrix
    // TODO: do this on CPU per mesh to save performance
    mat3 normalMatrix = transpose(inverse(mat3(model)));
    vs_out.normalMatrix = normalMatrix;

    // Transform tangent and normal to model's transform
    vec3 T = normalize(normalMatrix * aTangent);
    vec3 N = normalize(normalMatrix * aNormal);

    // TODO: could replace using normalMatrix with these to optimize
    // TODO: at the cost of visual clarity
    // vec3 T = normalize(vec3(model * vec4(aTangent, 0.0)));
    // vec3 N = normalize(vec3(model * vec4(aNormal, 0.0)));

    // T = normalize(T - dot(T, N) * N);

    // Calculate bitangent
    // vec3 B = cross(N, T);
    vec3 B = normalize(normalMatrix * aBitangent);
    // vec3 B = normalize(vec3(model * vec4(aBitangent, 0.0)));

    mat3 TBN = transpose(mat3(T, B, N));

    vs_out.TangentPointLightPosition = TBN * pointLightPosition;
    vs_out.TangentDirLightDir = TBN * dirLightDir;
    vs_out.TangentViewPos = TBN * viewPos;
    vs_out.TangentFragPos = TBN * fragPos;

    gl_Position = projection * view * model * vec4(aPos, 1.0);
}
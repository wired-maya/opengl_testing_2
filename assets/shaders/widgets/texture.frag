#version 330 core

out vec4 FragColor;
in vec2 TexCoords;

#define maxTextures 4

struct Material {
    int diffuseCount;
    sampler2D diffuse[maxTextures];
};

uniform Material material;

void main() {
    if (material.diffuseCount > 0) {
        FragColor = texture(material.diffuse[0], TexCoords);
    } else {
        discard;
    }
}
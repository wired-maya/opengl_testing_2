#version 330 core
out vec4 FragColor;
flat in int instanceID;

uniform vec4 BackgroundWidgets[16];

#define maxTextures 4

struct Material {
    int diffuseCount;
    sampler2D diffuse[maxTextures];
    vec3 diffuseFloat;

    int specularCount;
    sampler2D specular[maxTextures];
    vec3 specularFloat;
    
    int normalCount;
    sampler2D normal[maxTextures];

    int displacementCount;
    sampler2D displacement[maxTextures];

    int shininessCount;
    sampler2D shininess[maxTextures];
    float shininessFloat;
};

uniform Material material;

in vec2 TexCoords;

void main() {
    if (instanceID == 1) {
        FragColor = texture(material.diffuse[0], TexCoords);
    }
    else if (instanceID == 3) {
        FragColor = texture(material.diffuse[1], TexCoords);
    } else {
        vec4 color = BackgroundWidgets[instanceID];

        if (color.w == 0.0) discard;

        FragColor = color;
    }
}
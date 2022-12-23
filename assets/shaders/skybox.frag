#version 330 core

const float gamma = 2.2;

layout (location = 2) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

in vec3 TexCoords;

#define maxTextures 4

struct Material {
    int diffuseCount;
    samplerCube diffuse[maxTextures];
    
    float shininess;
};

uniform Material material;

void main()
{    
    FragColor = texture(material.diffuse[0], TexCoords);
    BrightColor = vec4(0.0); // Don't let skybox through into bloom layer

    // Gamma correction
    // FragColor.rgb = pow(FragColor.rgb, vec3(1.0 / gamma));
}
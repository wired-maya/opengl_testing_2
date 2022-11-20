#version 330 core

const float gamma = 2.2;

struct Material {
    samplerCube diffuse;
    samplerCube specular;
    float shininess;
};

layout (location = 2) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

in vec3 TexCoords;

uniform Material material;

void main()
{    
    FragColor = texture(material.diffuse, TexCoords);
    BrightColor = vec4(0.0); // Don't let skybox through into bloom layer

    // Gamma correction
    // FragColor.rgb = pow(FragColor.rgb, vec3(1.0 / gamma));
}
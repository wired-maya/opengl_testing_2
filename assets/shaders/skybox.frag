#version 330 core

const float gamma = 2.2;

struct Material {
    samplerCube diffuse;
    samplerCube specular;
    float shininess;
};

out vec4 FragColor;

in vec3 TexCoords;

uniform Material material;

void main()
{    
    FragColor = texture(material.diffuse, TexCoords);

    // Gamma correction
    FragColor.rgb = pow(FragColor.rgb, vec3(1.0 / gamma));
}
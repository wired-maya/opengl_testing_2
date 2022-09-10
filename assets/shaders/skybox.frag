#version 330 core

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
}
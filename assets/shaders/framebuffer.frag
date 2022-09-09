
#version 330 core
out vec4 FragColor;
  
in vec2 TexCoords;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

uniform Material material;

void main()
{ 
    // Post processing effects are done here
    // FragColor = texture(material.diffuse, TexCoords);

    // Inversion
    // FragColor = vec4(vec3(1.0 - texture(material.diffuse, TexCoords)), 1.0);

    // grayscale
    FragColor = texture(material.diffuse, TexCoords);
    float average = 0.2126 * FragColor.r + 0.7152 * FragColor.g + 0.0722 * FragColor.b;
    FragColor = vec4(average, average, average, 1.0);
}
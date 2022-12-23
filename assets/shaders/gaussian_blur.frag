#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

#define maxTextures 4

struct Material {
    int diffuseCount;
    sampler2D diffuse[maxTextures];

    int specularCount;
    sampler2D specular[maxTextures];
    
    int normalCount;
    sampler2D normal[maxTextures];

    int displacementCount;
    sampler2D displacement[maxTextures];
    
    float shininess;
};

uniform Material material;

uniform bool horizontal;
// TODO: find a way to change intensity based on HDR colour value?
uniform float weight[5] = float[] (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
    vec2 tex_offset = 1.0 / textureSize(material.diffuse[0], 0); // Gets size of single texel
    vec3 result = texture(material.diffuse[0], TexCoords).rgb * weight[0]; // Current fragment's contribution
    if (horizontal) {
        for (int i = 1; i < 5; ++i) {
            result += texture(material.diffuse[0], TexCoords + vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
            result += texture(material.diffuse[0], TexCoords - vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
        }
    }
    else {
        for (int i = 1; i < 5; ++i) {
            result += texture(material.diffuse[0], TexCoords + vec2(0.0, tex_offset.y * i)).rgb * weight[i];
            result += texture(material.diffuse[0], TexCoords - vec2(0.0, tex_offset.y * i)).rgb * weight[i];
        }
    }

    FragColor = vec4(result, 1.0);
}
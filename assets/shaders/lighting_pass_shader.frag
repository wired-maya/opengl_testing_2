#version 330 core

layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

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

// TODO: Include light type?
// TODO: Have this be a location thing like model transform
struct Light {
    vec3 Position;
    vec3 Color;
    float Radius;
};

in vec2 TexCoords;

const int NR_LIGHTS = 32;
uniform Light lights[NR_LIGHTS];
uniform vec3 viewPos;

void main() {
    // For forward shaded light fragments
    if (!(material.diffuseCount > 0)) {
        // TODO: find a way to get index to get light colour
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        BrightColor = vec4(1.0);
        return;
    }

    // Get data from G-Buffer
    vec3 FragPos = texture(material.diffuse[0], TexCoords).rgb;
    vec3 Normal = texture(material.diffuse[1], TexCoords).rgb;
    
    vec4 AlbedoSpec = texture(material.diffuse[2], TexCoords);
    vec3 Albedo = AlbedoSpec.rgb;
    float Specular = AlbedoSpec.a;

    // Calculate lighting
    vec3 lighting = Albedo * 0.1; // Hard coded ambient component
    vec3 viewDir = normalize(viewPos - FragPos);
    for (int i = 0; i < NR_LIGHTS; ++i) {
        // Check if you are outside the radius or not and don't do lighting calcs if you are

        float distance = length(lights[i].Position - FragPos);
        if (distance < lights[i].Radius) {
            // Diffuse
            vec3 lightDir = normalize(lights[i].Position - FragPos);
            vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Albedo * lights[i].Color;
            lighting += diffuse;
        }
    }

    FragColor = vec4(lighting, 1.0);
    BrightColor = vec4(0.0, 0.0, 0.0, 1.0);
}
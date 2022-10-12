#version 330 core

layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

struct Material {
    bool has_diffuse;
    sampler2D diffuse;

    bool has_specular;
    sampler2D specular;
    
    bool has_normal;
    sampler2D normal;

    bool has_displacement;
    sampler2D displacement;
    
    float shininess;
};

// TODO: Include light type?
// TODO: Have this be a location thing like model transform
struct Light {
    vec3 Position;
    vec3 Color;
};

in vec2 TexCoords;

const int NR_LIGHTS = 32;
uniform Light lights[NR_LIGHTS];
uniform Material material;
uniform vec3 viewPos;

void main() {
    // For forward shaded light fragments
    if (!material.has_diffuse) {
        // TODO: find a way to get index to get light colour
        FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        BrightColor = vec4(1.0);
        return;
    }

    // Get data from G-Buffer
    vec3 FragPos = texture(material.diffuse, TexCoords).rgb;
    vec3 Normal = texture(material.specular, TexCoords).rgb;
    
    vec4 AlbedoSpec = texture(material.normal, TexCoords);
    vec3 Albedo = AlbedoSpec.rgb;
    float Specular = AlbedoSpec.a;

    // Calculate lighting
    vec3 lighting = Albedo * 0.1; // Hard coded ambient component
    vec3 viewDir = normalize(viewPos - FragPos);
    for (int i = 0; i < NR_LIGHTS; ++i) {
        // Diffuse
        vec3 lightDir = normalize(lights[i].Position - FragPos);
        vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Albedo * lights[i].Color;
        lighting += diffuse;
    }

    FragColor = vec4(lighting, 1.0);
    BrightColor = vec4(0.0, 0.0, 0.0, 1.0);
}
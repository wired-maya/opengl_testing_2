#version 330 core
layout (location = 0) out vec3 gPosition;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec4 gAlbedoSpec;

const float gamma = 2.2;

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

in VS_OUT {
    vec2 texCoord;
    vec4 FragPosLightSpace;
    vec3 TangentPointLightPosition;
    vec3 TangentDirLightDir;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    vec3 fragPos;
    vec3 lightPos;
    vec3 viewPos;
    vec3 Normal;
} fg_in;

uniform Material material;

void main() {
    // TODO: could use parallax mapping to manipulate position in framebuffer!!!
    // Store frag pos vector in the first gbuffer texture
    gPosition = fg_in.fragPos;

    // TODO: all of these just have to be in the same coord space, so you could
    // TODO: easily implement normal mapping
    // Store per-fragment normals to gbuffer
    gNormal = fg_in.Normal;

    // Store diffuse per-fragment color
    gAlbedoSpec.rgb = material.has_diffuse ? texture(material.diffuse, fg_in.texCoord).rgb : vec3(0.0);

    // Store specular intensity in the alpha component of gAlbedoSpec
    gAlbedoSpec.a = material.has_specular ? texture(material.specular, fg_in.texCoord).r : 1.0;
}
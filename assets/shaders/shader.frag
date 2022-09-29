#version 330 core

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

struct DirLight {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;

    float far_plane;
};

// struct SpotLight {
//     vec3 position;
//     vec3 direction;
    
//     float cutOff;
//     float outerCutOff;

//     float constant;
//     float linear;
//     float quadratic;

//     vec3 ambient;
//     vec3 diffuse;
//     vec3 specular;
// };

layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

in GS_OUT {
    vec2 texCoord;
    vec4 FragPosLightSpace;
    vec3 TangentPointLightPosition;
    vec3 TangentDirLightDir;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
    vec3 fragPos;
    vec3 lightPos;
    vec3 viewPos;
} fg_in;

uniform Material material;
uniform DirLight dirLight;
#define NR_POINT_LIGHTS 1
uniform PointLight pointLight;
// uniform SpotLight spotLight;

uniform samplerCube skybox;
uniform sampler2D shadowMap;
uniform samplerCube shadowCubeMap;

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir, vec3 lightPos, vec3 fragPos, vec2 texCoord);
vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, vec3 lightPos, vec2 texCoord);
// vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec4 CalcReflection(vec3 normal, vec3 fragPos, vec3 viewPos);
vec4 CalcRefraction(vec3 normal, vec3 fragPos, vec3 viewPos, float ratio);
float ShadowCalculation(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir);
float CubeShadowCalculation(vec3 fragPos, vec3 normal, vec3 lightPos, float far_plane);
vec2 ParallaxMapping(vec2 texCoord, vec3 viewDir);

// LOTS of room for optimization:
//   There are lot of duplicated calculations in this approach spread out over the light type functions (e.g. calculating the reflect vector, diffuse and specular terms, and sampling the material textures) so there's room for optimization here. 
void main() {
    // If this has no diffuse, make it a super bright object
    if (!material.has_diffuse) {
        // TODO: Find a way to differentiate between light sources
        FragColor = vec4(dirLight.diffuse, 1.0);
        return;
    }

    // Properties
    vec3 normal;

    // If normal map is present, use it
    if (material.has_normal) {
        // Obtain normal from normal map in range [0,1]
        normal = texture(material.normal, fg_in.texCoord).rgb;
        // Transform normal vector to range [-1,1]
        normal = normalize(normal * 2.0 - 1.0);
    }
    else normal = vec3(0.0, 0.0, 1.0);

    // normal = vec3(0.0, 0.0, 1.0);

    vec3 viewDir = normalize(fg_in.TangentViewPos - fg_in.TangentFragPos);

    vec2 texCoord;

    // If depth map is present, use it
    if (material.has_displacement) {
        texCoord = ParallaxMapping(fg_in.texCoord, viewDir);
        if (texCoord.x > 1.0 || texCoord.y > 1.0 || texCoord.x < 0.0 || texCoord.y < 0.0) discard;
    }
    else texCoord = fg_in.texCoord;

    vec4 result = CalcDirLight(dirLight, normal, viewDir, fg_in.TangentDirLightDir, fg_in.TangentFragPos, texCoord);
    // vec4 result = CalcPointLight(pointLight, normal, fg_in.TangentFragPos, viewDir, fg_in.TangentPointLightPosition, texCoord);
    // result += CalcPointLight(pointLights[0], fg_in.Normal, fg_in.fragPos, viewDir);

    FragColor = vec4(result.rgb, 1.0);

    // TODO: fix alpha blending
    // FragColor = result;
    // Gamma correction
    // FragColor.rgb = pow(FragColor.rgb, vec3(1.0 / gamma));

    // FragColor = CalcReflection(norm, fragPos, viewPos);
    // FragColor = CalcRefraction(norm, fragPos, viewPos, 1.00 / 1.33); // Refraction ratio for water

    float brightness = dot(FragColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    if (brightness > 1.0) BrightColor = vec4(FragColor.rgb, 1.0);
    else BrightColor = vec4(0.0, 0.0, 0.0, 1.0);
}

vec2 ParallaxMapping(vec2 texCoord, vec3 viewDir) {
    // float height = texture(material.displacement, texCoord).r;
    // vec2 p = (viewDir.xy / viewDir.z) * (height * 0.1); // Height scale replaced with double for now
    // return texCoord - p;

    // Number of depth layers
    const float minLayers = 8.0;
    const float maxLayers = 32.0;
    float numLayers = mix(maxLayers, minLayers, abs(dot(vec3(0.0, 0.0, 1.0), viewDir)));

    // Scale how deep the effect goes
    const float height_scale = 0.1;
    // Calculate the size of each layer
    float layerDepth = 1.0 / numLayers;
    // Depth of current layer
    float currentLayerDepth = 0.0;
    // The amount to shift the texture coordinates per layer (from vector P)
    vec2 P = viewDir.xy * height_scale;
    vec2 deltaTexCoords = P / numLayers;

    // Get initial values
    vec2 currentTexCoords = texCoord;
    float currentDepthMapValue = texture(material.displacement, currentTexCoords).r;

    while (currentLayerDepth < currentDepthMapValue) {
        // Shift texture coords along direction of P
        currentTexCoords -= deltaTexCoords;
        // Get depthmap value at current texture coordinates
        currentDepthMapValue = texture(material.displacement, currentTexCoords).r;
        // Get depth of next layer
        currentLayerDepth += layerDepth;
    }

    // Get texture coords before collision
    vec2 prevTexCoords = currentTexCoords + deltaTexCoords;

    // Get depth after and before collision for linear interpolation
    float afterDepth = currentDepthMapValue - currentLayerDepth;
    float beforeDepth = texture(material.displacement, prevTexCoords).r - currentLayerDepth + layerDepth;

    // Interpolation of texture coordinates
    float weight = afterDepth / (afterDepth - beforeDepth);
    vec2 finalTexCoords = prevTexCoords * weight + currentTexCoords * (1.0 - weight);

    return finalTexCoords;
}

float ShadowCalculation(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir) {
    // Perform persepective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // Transform to range [0,1] like the depth map
    projCoords = (projCoords * 0.5) + 0.5;

    float currentDepth = projCoords.z;

    float bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.005);
    
    // PCF
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;

    if (projCoords.z > 1.0) shadow = 0.0;

    return shadow;
}

float CubeShadowCalculation(vec3 fragPos, vec3 normal, vec3 lightPos, float far_plane) {
    vec3 fragToLight = fg_in.fragPos - fg_in.lightPos;

    float currentDepth = length(fragToLight);

    vec3 sampleOffsetDirections[20] = vec3[](
        vec3( 1,  1,  1), vec3( 1, -1,  1), vec3(-1, -1,  1), vec3(-1,  1,  1), 
        vec3( 1,  1, -1), vec3( 1, -1, -1), vec3(-1, -1, -1), vec3(-1,  1, -1),
        vec3( 1,  1,  0), vec3( 1, -1,  0), vec3(-1, -1,  0), vec3(-1,  1,  0),
        vec3( 1,  0,  1), vec3(-1,  0,  1), vec3( 1,  0, -1), vec3(-1,  0, -1),
        vec3( 0,  1,  1), vec3( 0, -1,  1), vec3( 0, -1, -1), vec3( 0,  1, -1)
    );

    float shadow = 0.0;
    float bias = 0.15;
    int samples = 20;
    float viewDistance = length(fg_in.viewPos - fg_in.fragPos);
    float diskRadius = (1.0 + (viewDistance / far_plane)) / 25.0;

    for (int i = 0; i < samples; ++i) {
        float closestDepth = texture(shadowCubeMap, fragToLight + sampleOffsetDirections[i] * diskRadius).r;
        closestDepth *= far_plane; // Undo mapping [0,1]
        if (currentDepth - bias > closestDepth) shadow += 1.0;
    }
    
    shadow /= float(samples);

    return shadow;
}

// vec4 CalcReflection(vec3 normal, vec3 fragPos, vec3 viewPos) {
//     vec3 I = normalize(fragPos - viewPos);
//     vec3 R = reflect(I, normal);
//     return vec4(texture(skybox, R).rgb, 1.0);
// }

// vec4 CalcRefraction(vec3 normal, vec3 fragPos, vec3 viewPos, float ratio) {
//     vec3 I = normalize(fragPos - viewPos);
//     vec3 R = refract(I, normal, ratio);
//     return vec4(texture(skybox, R).rgb, 1.0);
// }

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir, vec3 lightPos, vec3 fragPos, vec2 texCoord) {
    // vec3 lightDir = normalize(-light.direction);
    // vec3 lightDir = normalize(lightPos - fragPos);
    vec3 lightDir = normalize(lightPos);
    // Diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular shading
    // vec3 reflectDir = reflect(-lightDir, normal);
    // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);

    // Retrieve texture values if they are present for the mesh
    vec4 diffuse_texel = material.has_diffuse ? texture(material.diffuse, texCoord) : vec4(1.0);
    vec4 specular_texel = material.has_specular ? texture(material.specular, texCoord) : vec4(1.0);

    // Combine results
    vec4 ambient = vec4(light.ambient, 1.0) * diffuse_texel;
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * diffuse_texel;
    vec4 specular = vec4(light.specular, 1.0) * spec * specular_texel;

    // Calculate shadows
    float shadow = ShadowCalculation(fg_in.FragPosLightSpace, normal, lightDir);

    return (ambient + ((1.0 - shadow) * (diffuse + specular)));
    // return (ambient + diffuse + specular);
}

vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, vec3 lightPos, vec2 texCoord) {
    vec3 lightDir = normalize(lightPos - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    // vec3 reflectDir = reflect(-lightDir, normal);
    // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // attenuation
    float distance = length(lightPos - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // Retrieve texture values if they are present for the mesh
    vec4 diffuse_texel = material.has_diffuse ? texture(material.diffuse, texCoord) : vec4(1.0);
    vec4 specular_texel = material.has_specular ? texture(material.specular, texCoord) : vec4(1.0);

    // Combine results
    vec4 ambient = vec4(light.ambient, 1.0) * diffuse_texel;
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * diffuse_texel;
    vec4 specular = vec4(light.specular, 1.0) * spec * specular_texel;

    // Add attenuation
    ambient *= vec4(vec3(attenuation), 1.0);
    diffuse *= vec4(vec3(attenuation), 1.0);
    specular *= vec4(vec3(attenuation), 1.0);

    // Calculate shadows
    float shadow = CubeShadowCalculation(fragPos, normal, lightPos, light.far_plane);

    return (ambient + ((1.0 - shadow) * (diffuse + specular)));
    // return (ambient + diffuse + specular);
}

// TODO: remake with new stuff
// vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
//     vec3 lightDir = normalize(light.position - fragPos);
//     // diffuse shading
//     float diff = max(dot(normal, lightDir), 0.0);
//     // specular shading
//     // vec3 reflectDir = reflect(-lightDir, normal);
//     // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
//     vec3 halfwayDir = normalize(lightDir + viewDir);
//     float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
//     // attenuation
//     float distance = length(light.position - fragPos);
//     float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
//     // spotlight intensity
//     float theta = dot(lightDir, normalize(-light.direction));
//     float epsilon = light.cutOff - light.outerCutOff;
//     float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
//     // Retrieve texture values if they are present for the mesh
//     vec4 diffuse_texel = material.has_diffuse ? texture(material.diffuse, fg_in.texCoord) : vec4(1.0);
//     vec4 specular_texel = material.has_specular ? texture(material.specular, fg_in.texCoord) : vec4(1.0);

//     // Combine results
//     vec4 ambient = vec4(light.ambient, 1.0) * diffuse_texel;
//     vec4 diffuse = vec4(light.diffuse, 1.0) * diff * diffuse_texel;
//     vec4 specular = vec4(light.specular, 1.0) * spec * specular_texel;
//     ambient *= attenuation * intensity;
//     diffuse *= attenuation * intensity;
//     specular *= attenuation * intensity;
//     return (ambient + diffuse + specular);
// }
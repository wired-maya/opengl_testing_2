#version 330 core

const float gamma = 2.2;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

struct DirLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;

    float far_plane;
};

struct SpotLight {
    vec3 position;
    vec3 direction;
    
    float cutOff;
    float outerCutOff;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

out vec4 FragColor;

in GS_OUT {
   vec2 texCoord;
   vec3 Normal;
   vec3 fragPos;
   vec4 FragPosLightSpace;
} fg_in;

uniform Material material;
uniform DirLight dirLight;
#define NR_POINT_LIGHTS 256
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform SpotLight spotLight;

uniform vec3 viewPos;
uniform samplerCube skybox;
uniform sampler2D shadowMap;
uniform samplerCube shadowCubeMap;

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir);
vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec4 CalcReflection(vec3 normal, vec3 fragPos, vec3 viewPos);
vec4 CalcRefraction(vec3 normal, vec3 fragPos, vec3 viewPos, float ratio);
float ShadowCalculation(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir);
float CubeShadowCalculation(vec3 fragPos, vec3 normal, vec3 lightPos, float far_plane);

// LOTS of room for optimization:
//   There are lot of duplicated calculations in this approach spread out over the light type functions (e.g. calculating the reflect vector, diffuse and specular terms, and sampling the material textures) so there's room for optimization here. 
void main() {
    // Properties
    // vec3 norm = normalize(fg_in.Normal);
    vec3 viewDir = normalize(viewPos - fg_in.fragPos);

    vec4 result = CalcDirLight(dirLight, fg_in.Normal, viewDir);
    // vec4 result = CalcPointLight(pointLights[0], fg_in.Normal, fg_in.fragPos, viewDir);
    // for (int i = 0; i < NR_POINT_LIGHTS; i++) {
    //     result += CalcPointLight(pointLights[i], fg_in.Normal, fg_in.fragPos, viewDir);
    // }
    // result += CalcSpotLight(spotLight, norm, fragPos, viewDir);
    result += CalcPointLight(pointLights[0], fg_in.Normal, fg_in.fragPos, viewDir);

    FragColor = result;
    // Gamma correction
    FragColor.rgb = pow(FragColor.rgb, vec3(1.0 / gamma));

    // FragColor = CalcReflection(norm, fragPos, viewPos);
    // FragColor = CalcRefraction(norm, fragPos, viewPos, 1.00 / 1.33); // Refraction ratio for water
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
    vec3 fragToLight = fragPos - lightPos;

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
    float viewDistance = length(viewPos - fragPos);
    float diskRadius = (1.0 + (viewDistance / far_plane)) / 25.0;

    for (int i = 0; i < samples; ++i) {
        float closestDepth = texture(shadowCubeMap, fragToLight + sampleOffsetDirections[i] * diskRadius).r;
        closestDepth *= far_plane; // Undo mapping [0,1]
        if (currentDepth - bias > closestDepth) shadow += 1.0;
    }
    
    shadow /= float(samples);

    return shadow;
}

vec4 CalcReflection(vec3 normal, vec3 fragPos, vec3 viewPos) {
    vec3 I = normalize(fragPos - viewPos);
    vec3 R = reflect(I, normal);
    return vec4(texture(skybox, R).rgb, 1.0);
}

vec4 CalcRefraction(vec3 normal, vec3 fragPos, vec3 viewPos, float ratio) {
    vec3 I = normalize(fragPos - viewPos);
    vec3 R = refract(I, normal, ratio);
    return vec4(texture(skybox, R).rgb, 1.0);
}

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir) {
    // vec3 lightDir = normalize(-light.direction);
    vec3 lightDir = normalize(light.direction - fg_in.fragPos);
    // Diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular shading
    // vec3 reflectDir = reflect(-lightDir, normal);
    // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // Combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, fg_in.texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, fg_in.texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.specular, fg_in.texCoord);

    // Calculate shadows
    float shadow = ShadowCalculation(fg_in.FragPosLightSpace, normal, lightDir);

    return (ambient + ((1.0 - shadow) * (diffuse + specular)));
    // return vec4(shadow);
}

vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    // vec3 reflectDir = reflect(-lightDir, normal);
    // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, fg_in.texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, fg_in.texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.specular, fg_in.texCoord);
    // ambient *= attenuation;
    // diffuse *= attenuation;
    // specular *= attenuation;

    // Calculate shadows
    float shadow = CubeShadowCalculation(fragPos, normal, light.position, light.far_plane);

    return (ambient + ((1.0 - shadow) * (diffuse + specular)));
    // return (ambient + diffuse + specular);
}

vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    // vec3 reflectDir = reflect(-lightDir, normal);
    // float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // spotlight intensity
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
    // combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, fg_in.texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, fg_in.texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.specular, fg_in.texCoord);
    ambient *= attenuation * intensity;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    return (ambient + diffuse + specular);
}
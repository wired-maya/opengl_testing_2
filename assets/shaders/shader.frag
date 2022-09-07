#version 330 core

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

in vec2 texCoord;
in vec3 Normal;
in vec3 fragPos;

uniform Material material;
uniform DirLight dirLight;
#define NR_POINT_LIGHTS 4
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform SpotLight spotLight;

uniform vec3 viewPos;

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir);
vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);

// LOTS of room for optimization:
//   There are lot of duplicated calculations in this approach spread out over the light type functions (e.g. calculating the reflect vector, diffuse and specular terms, and sampling the material textures) so there's room for optimization here. 
void main() {
    // Properties
    vec3 norm = normalize(Normal);
    vec3 viewDir = normalize(viewPos - fragPos);

    vec4 result = CalcDirLight(dirLight, norm, viewDir);
    for (int i = 0; i < NR_POINT_LIGHTS; i++) {
        result += CalcPointLight(pointLights[i], norm, fragPos, viewDir);
    }
    // result += CalcSpotLight(spotLight, norm, fragPos, viewDir);

    if(result.a < 0.1) discard;
    FragColor = result;
}

vec4 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    // Diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // Combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.diffuse, texCoord);
    return (ambient + diffuse + specular);
}

vec4 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + 
  			     light.quadratic * (distance * distance));    
    // combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.diffuse, texCoord);
    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;
    return (ambient + diffuse + specular);
}

vec4 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // attenuation
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
    // spotlight intensity
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
    // combine results
    vec4 ambient = vec4(light.ambient, 1.0) * texture(material.diffuse, texCoord);
    vec4 diffuse = vec4(light.diffuse, 1.0) * diff * texture(material.diffuse, texCoord);
    vec4 specular = vec4(light.specular, 1.0) * spec * texture(material.diffuse, texCoord);
    ambient *= attenuation * intensity;
    diffuse *= attenuation * intensity;
    specular *= attenuation * intensity;
    return (ambient + diffuse + specular);
}
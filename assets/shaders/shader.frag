#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

struct Light {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

out vec4 FragColor;

in vec2 texCoord;
in vec3 Normal;
in vec3 fragPos;

uniform Material material;
uniform Light light;

uniform sampler2D texture1;
uniform sampler2D texture2;

uniform vec3 viewPos;

void main() {
    float ambientStrength = 0.1;
    vec3 ambient = light.ambient * material.ambient;

    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(light.position - fragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = (diff * material.diffuse) * light.diffuse;

    float specularStrength = 0.5;
    vec3 viewDir = normalize(viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess); // 32 here is the shininess of the object
    vec3 specular = (spec * material.specular) * light.specular;

    // FragColor = vec4(ambient + diffuse + specular, 1.0) * mix(texture(texture1, texCoord), texture(texture2, texCoord), 0.4);
    vec3 result = (ambient + diffuse + specular);
    FragColor = vec4(result, 1.0);
}
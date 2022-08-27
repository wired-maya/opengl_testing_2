#version 330 core

out vec4 FragColor;

in vec3 ourColor;
in vec2 texCoord;
in vec3 Normal;
in vec3 fragPos;

uniform sampler2D texture1;
uniform sampler2D texture2;

uniform vec3 lightColor;
uniform vec3 lightPos;

void main() {
    // FragColor = vec4(ourColor, 1.0f);
    // FragColor = mix(texture(texture1, texCoord), texture(texture2, texCoord), 0.4) * vec4(ourColor, 1.0f);
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - fragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    FragColor = vec4(ambient + diffuse, 1.0) * mix(texture(texture1, texCoord), texture(texture2, texCoord), 0.4);
}
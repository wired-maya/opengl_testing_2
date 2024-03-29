#version 330 core

out vec4 FragColor;
in vec2 TexCoords;

uniform vec4 colour;

void main() {
    if (colour.w == 0.0) discard;
    FragColor = colour;
}
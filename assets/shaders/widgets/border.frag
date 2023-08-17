#version 330 core

out vec4 FragColor;
in vec2 TexCoords;

uniform vec4 colour;
uniform vec4 border_widths;

void main() {
    if (
        TexCoords.x <= border_widths.x ||
        TexCoords.x >= (1.0 - border_widths.y) ||
        TexCoords.y <= border_widths.w ||
        TexCoords.y >= (1.0 - border_widths.z)
    ) {
        if (colour.w == 0.0) discard;
        FragColor = colour;
    } else discard;
}
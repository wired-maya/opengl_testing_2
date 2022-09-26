#version 330 core
out vec4 FragColor;

in GS_OUT {
    vec4 color;
} fg_in;

void main() {
    FragColor = fg_in.color;
}
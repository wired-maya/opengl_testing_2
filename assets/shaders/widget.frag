#version 330 core
out vec4 FragColor;

uniform vec4 BackgroundWidgets[16];

void main() {
    FragColor = BackgroundWidgets[0];
}
#version 330 core
out vec4 FragColor;
flat in int instanceID;

uniform vec4 BackgroundWidgets[16];

void main() {
    vec4 color = BackgroundWidgets[instanceID];

    if (color.w == 0.0) discard;

    FragColor = color;
}
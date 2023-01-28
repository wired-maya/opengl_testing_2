#version 330 core
out vec4 FragColor;
flat in int instanceID;

uniform vec4 BackgroundWidgets[16];

void main() {
    FragColor = BackgroundWidgets[instanceID];
}
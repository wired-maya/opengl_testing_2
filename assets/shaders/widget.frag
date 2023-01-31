#version 460 core
out vec4 FragColor;
in vec2 TexCoords;
flat in int instanceID;

// In place of Enums
const int Background = 1;
const int Texture = 2;
const int Border = 3;

// The index corresponds to the primitive's own info array
struct Widget {
    int type;
    int index;
};

uniform Widget widgets[64];

uniform vec4 backgroundWidgets[16];

#define maxTextures 16
struct Material {
    int diffuseCount;
    sampler2D diffuse[maxTextures];
};
uniform Material material;

void main() {
    Widget widget = widgets[instanceID];

    switch (widget.type) {
        case Background:
            vec4 color = backgroundWidgets[widget.index];
            if (color.w == 0.0) discard;
            FragColor = color;
            break;
        case Texture:
            FragColor = texture(material.diffuse[widget.index], TexCoords);
            break;
        case Border:
            discard;
            break;
    }
}
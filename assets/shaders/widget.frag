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

struct BorderWidget {
    vec4 colour;
    vec4 widths;
};
uniform BorderWidget borderWidgets[16];

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
            // TODO: Implement
            // TODO: Use tex coords with reach edge of vec4 widths to see which fragments
            // TODO: you draw and which you don't
            BorderWidget widget = borderWidgets[widget.index];

            if (
                TexCoords.x <= widget.widths.x ||
                TexCoords.x >= (1.0 - widget.widths.y) ||
                TexCoords.y <= widget.widths.w ||
                TexCoords.y >= (1.0 - widget.widths.z)
            ) {
                FragColor = widget.colour;
            } else discard;
            break;
    }
}
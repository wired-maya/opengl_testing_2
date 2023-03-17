#version 460 core
// #extension GL_ARB_bindless_texture : require
out vec4 FragColor;
in vec2 TexCoords;
flat in int instanceID;
layout (std430, binding = 0) buffer DataBufferArray {
    uint dataBufferArray[];
};

// In place of Enums
const int Background = 1;
const int Texture = 2;
const int Border = 3;

void main() {
    int offset = instanceID * 256;
    uint type = dataBufferArray[offset];

    switch (type) {
        case Background:
            vec4 color = vec4(
                uintBitsToFloat(dataBufferArray[offset + 1]),
                uintBitsToFloat(dataBufferArray[offset + 2]),
                uintBitsToFloat(dataBufferArray[offset + 3]),
                uintBitsToFloat(dataBufferArray[offset + 4])
            );
            if (color.w == 0.0) discard;
            FragColor = color;
            break;
        case Texture:
            // sampler2D tex = sampler2D(
            //     uvec2(
            //         dataBufferArray[offset + 1],
            //         dataBufferArray[offset + 2]
            //     )
            // );
            // FragColor = texture(tex, -TexCoords);
            discard;
            
            break;
        case Border:
            vec4 border_widths = vec4(
                uintBitsToFloat(dataBufferArray[offset + 5]),
                uintBitsToFloat(dataBufferArray[offset + 6]),
                uintBitsToFloat(dataBufferArray[offset + 7]),
                uintBitsToFloat(dataBufferArray[offset + 8])
            );

            if (
                TexCoords.x <= border_widths.x ||
                TexCoords.x >= (1.0 - border_widths.y) ||
                TexCoords.y <= border_widths.w ||
                TexCoords.y >= (1.0 - border_widths.z)
            ) {
                vec4 color = vec4(
                    uintBitsToFloat(dataBufferArray[offset + 1]),
                    uintBitsToFloat(dataBufferArray[offset + 2]),
                    uintBitsToFloat(dataBufferArray[offset + 3]),
                    uintBitsToFloat(dataBufferArray[offset + 4])
                );
                if (color.w == 0.0) discard;
                FragColor = color;
            } else discard;
            break;
    }
}
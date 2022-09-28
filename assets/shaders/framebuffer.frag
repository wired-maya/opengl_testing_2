#version 330 core
layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

in vec2 TexCoords;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

uniform Material material;

uniform float exposure;

const float offset = 1.0 / 300.0;

void main()
{ 
    // Post processing effects are done here

    // No effect
    // FragColor = texture(material.diffuse, TexCoords);

    // HDR tone mapping
    vec3 hdrColor = texture(material.diffuse, TexCoords).rgb;
    // Reinhard tone mapping
    // vec3 mapped = hdrColor / (hdrColor + vec3(1.0));
    // Exposure tone mapping
    vec3 mapped = vec3(1.0) - exp(-hdrColor * exposure);

    // Gamma correction
    const float gamma = 2.2;
    mapped = pow(mapped, vec3(1.0 / gamma));

    FragColor = vec4(mapped, 1.0);

    // Check whether frag output is higher than threshold, if so output as brightness colour
    float brightness = dot(FragColor.rgb, vec3(0.2126, 0.7152, 0.0722));
    if (brightness > 1.0) BrightColor = vec4(FragColor.rgb, 1.0);
    else BrightColor = vec4(0.0, 0.0, 0.0, 1.0);

    // Inversion
    // FragColor = vec4(vec3(1.0 - texture(material.diffuse, TexCoords)), 1.0);

    // Grayscale
    // FragColor = texture(material.diffuse, TexCoords);
    // float average = 0.2126 * FragColor.r + 0.7152 * FragColor.g + 0.0722 * FragColor.b;
    // FragColor = vec4(average, average, average, 1.0);

    // Kernel
    // vec2 offsets[9] = vec2[] (
    //     vec2(-offset,  offset), // top-left
    //     vec2( 0.0f,    offset), // top-center
    //     vec2( offset,  offset), // top-right
    //     vec2(-offset,  0.0f),   // center-left
    //     vec2( 0.0f,    0.0f),   // center-center
    //     vec2( offset,  0.0f),   // center-right
    //     vec2(-offset, -offset), // bottom-left
    //     vec2( 0.0f,   -offset), // bottom-center
    //     vec2( offset, -offset)  // bottom-right    
    // );

    // float kernel[9] = float[] (
    //     1, 1, 1,
    //     1, -8, 1,
    //     1, 1, 1
    // );

    // vec3 sampleTex[9];
    
    // for(int i = 0; i < 9; i++)
    // {
    //     sampleTex[i] = vec3(texture(material.diffuse, TexCoords.st + offsets[i]));
    // }
    // vec3 col = vec3(0.0);
    // for(int i = 0; i < 9; i++)
    //     col += sampleTex[i] * kernel[i];
    
    // FragColor = vec4(col, 1.0);

    // TODO: Figure out how to run gamma correction only on framebuffer instead of in
    // TODO: model and skybox shaders
}
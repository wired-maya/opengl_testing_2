#version 330 core
layout (location = 0) in vec3 aPos;
layout (std140) uniform Matrices {
    mat4 projection;
    mat4 view;
};

out vec3 TexCoords;

void main()
{
    // Remove translation from view matrix but only for the skybox
    mat4 viewDir = mat4(mat3(view));

    TexCoords = aPos;
    vec4 pos = projection * viewDir * vec4(aPos, 1.0);
    gl_Position = pos.xyww;
}
#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 18) out;
layout (std140) uniform CameraMatrices {
    mat4 projection;
    mat4 view;
};

in VS_OUT {
    vec3 normal;
    vec3 tangent;
    vec3 bitangent;
} gs_in[];

out GS_OUT {
    vec4 color;
} gs_out;

const float MAGNITUDE = 0.1;

void GenerateLines(int index) {
    gl_Position = projection * gl_in[index].gl_Position;
    gs_out.color = vec4(0.0, 0.0, 1.0, 1.0);
    EmitVertex();
    gl_Position = projection * (gl_in[index].gl_Position + vec4(gs_in[index].normal, 0.0) * MAGNITUDE);
    gs_out.color = vec4(0.0, 0.0, 1.0, 1.0);
    EmitVertex();
    EndPrimitive();

    gl_Position = projection * gl_in[index].gl_Position;
    gs_out.color = vec4(0.0, 1.0, 0.0, 1.0);
    EmitVertex();
    gl_Position = projection * (gl_in[index].gl_Position + vec4(gs_in[index].tangent, 0.0) * MAGNITUDE);
    gs_out.color = vec4(0.0, 1.0, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();

    gl_Position = projection * gl_in[index].gl_Position;
    gs_out.color = vec4(1.0, 0.0, 0.0, 1.0);
    EmitVertex();
    gl_Position = projection * (gl_in[index].gl_Position + vec4(gs_in[index].bitangent, 0.0) * MAGNITUDE);
    gs_out.color = vec4(1.0, 0.0, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
}

void main() {
    GenerateLines(0);
    GenerateLines(1);
    GenerateLines(2);
}
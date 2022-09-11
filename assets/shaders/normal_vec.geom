#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 6) out;
layout (std140) uniform Matrices {
    mat4 projection;
    mat4 view;
};

#define PRIMITIVE_LENGTH 3

in VS_OUT {
   vec2 texCoord;
   vec3 Normal;
   vec3 fragPos;
} gs_in[PRIMITIVE_LENGTH];

const float MAGNITUDE = 0.4;

void main() {
   for (int i = 0; i < 1; i++) {
      gl_Position = gl_in[i].gl_Position;
      
      EmitVertex();

      gl_Position = gl_in[i].gl_Position + vec4(gs_in[i].Normal, 0.0) * MAGNITUDE;
      
      EmitVertex();
      EndPrimitive();
   }
}  
#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

#define PRIMITIVE_LENGTH 3

in VS_OUT {
   vec2 texCoord;
   vec3 Normal;
   vec3 fragPos;
} gs_in[PRIMITIVE_LENGTH];

out GS_OUT {
   vec2 texCoord;
   vec3 Normal;
   vec3 fragPos;
} gs_out;

void main() {
   for (int i = 0; i < 3; i++) {
      gl_Position = gl_in[i].gl_Position;

      gs_out.texCoord = gs_in[i].texCoord;
      gs_out.Normal = gs_in[i].Normal;
      gs_out.fragPos = gs_in[i].fragPos;
      
      EmitVertex();
   }

   EndPrimitive();
}  
#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

#define PRIMITIVE_LENGTH 3

in VS_OUT {
   vec2 texCoord;
   vec4 FragPosLightSpace;
   vec3 TangentPointLightPosition;
   vec3 TangentDirLightDir;
   vec3 TangentViewPos;
   vec3 TangentFragPos;
   mat3 normalMatrix;
} gs_in[PRIMITIVE_LENGTH];

out GS_OUT {
   vec2 texCoord;
   vec4 FragPosLightSpace;
   vec3 TangentPointLightPosition;
   vec3 TangentDirLightDir;
   vec3 TangentViewPos;
   vec3 TangentFragPos;
   mat3 normalMatrix;
} gs_out;

uniform float time;

vec4 explode(vec4 position, vec3 normal)
{
   float magnitude = 2.0;
   vec3 direction = normal * ((sin(time) + 1.0) / 2.0) * magnitude;
   return position + vec4(direction, 0.0);
}

vec3 GetNormal()
{
   vec3 a = vec3(gl_in[0].gl_Position) - vec3(gl_in[1].gl_Position);
   vec3 b = vec3(gl_in[2].gl_Position) - vec3(gl_in[1].gl_Position);
   return normalize(cross(a, b));
}  

void main() {
   // vec3 normal = GetNormal();

   for (int i = 0; i < 3; i++) {
      // gl_Position = explode(gl_in[i].gl_Position, gs_in[i].Normal);
      gl_Position = gl_in[i].gl_Position;

      gs_out.texCoord = gs_in[i].texCoord;
      gs_out.FragPosLightSpace = gs_in[i].FragPosLightSpace;
      gs_out.TangentPointLightPosition = gs_in[i].TangentPointLightPosition;
      gs_out.TangentDirLightDir = gs_in[i].TangentDirLightDir;
      gs_out.TangentViewPos = gs_in[i].TangentViewPos;
      gs_out.TangentFragPos = gs_in[i].TangentFragPos;
      gs_out.normalMatrix = gs_in[i].normalMatrix;
      
      EmitVertex();
   }

   EndPrimitive();
}  
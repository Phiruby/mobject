#version 450
layout (quads, equal_spacing, cw) in;
layout (location = 0) in vec2 textureCoord[];
layout (location = 1) in vec3 fragmentColors[];
layout (location = 2) in vec3 fragNormals[];
layout (location = 3) in vec3 worldPos[];

layout(set = 0, binding = 0) uniform SceneBufferObject {
  vec3 camera_position;
  mat4 view;
  mat4 proj;
} sbo;

layout(set = 1, binding = 0) uniform UniformBufferObject {
    mat4 model;
} ubo;

layout (location = 0) out vec3 fragColor;
layout (location = 1) out vec2 fragTexCoord;
layout (location = 2) out vec3 outNormal;
layout (location = 3) out vec3 outPos;

float B2_0(float v) {return pow(1.0 - v, 2.0);}
float B2_1(float v) {return 2.0 * v * (1.0 - v);}
float B2_2(float v) {return pow(v, 2.0);}

void main() {
  float u = gl_TessCoord.x;
  float v = gl_TessCoord.y;
  float[] Bv = float[](B2_0(v), B2_1(v), B2_2(v));
  float[] Bu = float[](B2_0(u), B2_1(u), B2_2(u));
  int idx = 0;
  vec3 out_pos = vec3(0.0);
  vec2 out_tex_coord = vec2(0.0);
  vec3 out_frag_color = vec3(0.0);
  vec3 out_normal = vec3(0.0);
  vec3 world_pos = vec3(0.0);
  for (int i = 0; i < 3; i++) {
    for (int j = 0; j < 3; j++) {
      out_pos += Bv[i] * Bu[j] * gl_in[idx].gl_Position.xyz;
      out_tex_coord += Bv[i] * Bu[j] * textureCoord[idx];
      out_frag_color += Bv[i] * Bu[j] * fragmentColors[idx];
      out_normal += Bv[i] * Bu[j] * fragNormals[idx];
      world_pos += Bv[i] * Bu[j] * worldPos[idx];
      idx++;
    }
  }

  gl_Position = sbo.proj * sbo.view * ubo.model * vec4(out_pos, 1.0);
  fragColor = vec3(1.0, 1.0, 0.0);
  fragTexCoord = out_tex_coord;
  outNormal =  mat3(transpose(inverse(ubo.model))) * out_normal;
  outPos = world_pos;
}

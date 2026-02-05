#version 450
layout (quads, equal_spacing, cw) in;
layout (location = 0) in vec2 textureCoord[];
layout (location = 1) in vec3 fragmentColors[];

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

float B2_0(float v) {return pow(1.0 - v, 2.0);}
float B2_1(float v) {return 2.0 * v * (1.0 - v);}
float B2_2(float v) {return pow(v, 2.0);}

void main() {
  // take average position
  vec3 bary = gl_TessCoord;
  vec4 position = (bary.x * gl_in[0].gl_Position + bary.y * gl_in[1].gl_Position + bary.z * gl_in[2].gl_Position);
  fragColor = (bary.x * fragmentColors[0] + bary.y * fragmentColors[1] + bary.z * fragmentColors[2]);
  fragTexCoord = (bary.x * textureCoord[0] + bary.y * textureCoord[1] + bary.z * textureCoord[2]);
  gl_Position = sbo.proj * sbo.view * ubo.model * position;
  // float u = gl_TessCoord.x;
  // float v = gl_TessCoord.y;
  // float[] Bv = float[](B2_0(v), B2_1(v), B2_2(v));
  // float[] Bu = float[](B2_0(u), B2_1(u), B2_2(u));
  // int idx = 0;
  // vec3 out_pos = vec3(0.0);
  // vec2 out_tex_coord = vec2(0.0);
  // vec3 out_frag_color = vec3(0.0);
  // for (int i = 0; i < 3; i++) {
  //   for (int j = 0; j < 3; j++) {
  //     out_pos += Bv[i] * Bu[j] * gl_in[idx].gl_Position.xyz;
  //     out_tex_coord += Bv[i] * Bu[j] * textureCoord[idx];
  //     out_frag_color += Bv[i] * Bu[j] * fragmentColors[idx];
  //     idx++;
  //   }
  // }
  //
  // gl_Position = sbo.proj * sbo.view * ubo.model * vec4(out_pos, 1.0);
  // fragColor = vec3(1.0, 1.0, 0.0);
  // fragTexCoord = out_tex_coord;

}

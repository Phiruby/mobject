#version 450
layout (triangles, equal_spacing, cw) in;
layout (location = 0) in vec2 textureCoord[];
layout (location = 1) in vec3 fragmentColors[];

layout (location = 0) out vec3 fragColor;
layout (location = 1) out vec2 fragTexCoord;

void main() {
  // take average position
  vec3 bary = gl_TessCoord;
  vec4 position = (bary.x * gl_in[0].gl_Position + bary.y * gl_in[1].gl_Position + bary.z * gl_in[2].gl_Position);
  fragColor = (bary.x * fragmentColors[0] + bary.y * fragmentColors[1] + bary.z * fragmentColors[2]);
  fragTexCoord = (bary.x * textureCoord[0] + bary.y * textureCoord[1] + bary.z * textureCoord[2]);
  gl_Position = position;
}

#version 450
layout (triangles, equal_spacing, ccw) in;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout (location = 0) in vec2 textureCoord[];
layout (location = 1) in vec3 fragmentColors[];

layout (location = 0) out vec3 fragColor;
layout (location = 1) out vec2 fragTexCoord;

void main() {
  // take average position
  vec4 position = (gl_in[0].gl_Position + gl_in[1].gl_Position + gl_in[2].gl_Position) / 3;
  fragColor = (fragmentColors[0] + fragmentColors[1] + fragmentColors[2]) / 3;
  fragTexCoord = (textureCoord[0] + textureCoord[1] + textureCoord[2]) / 3;
  gl_Position = ubo.proj * ubo.model * ubo.view * position;
}

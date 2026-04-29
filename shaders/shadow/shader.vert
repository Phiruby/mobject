#version 450
// layout (location = 0) in vec3 aPos;
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inTexCoord;

layout(set = 0, binding = 0) uniform SceneBufferObject {
  vec3 camera_position;
  mat4 view;
  mat4 proj;
  mat4 lightSpaceMatrix;
} sbo;

layout(set = 1, binding = 0) uniform UniformBufferObject {
    mat4 model;
} ubo;


void main()
{
    gl_Position = sbo.lightSpaceMatrix * ubo.model * vec4(aPos, 1.0);
}

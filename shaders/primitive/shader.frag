#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(push_constant, std430) uniform pc {
  uint sampler_idx;
};
layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D texSampler[];

void main() {
  outColor = texture(texSampler[sampler_idx], fragTexCoord) * vec4(fragColor, 1.0);
}

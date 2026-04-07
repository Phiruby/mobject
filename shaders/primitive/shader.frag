#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(push_constant, std430) uniform pc {
  uint sampler_idx;
};
layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 inPos;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D texSampler[];

void main() {
  float ambientStrength = 0.1;
  vec3 lightColor = vec3(0.7, 0.7, 0.7);
  vec3 lightPos = vec3(10.0, 0.0, 0.0);

  vec3 amient = ambientStrength * lightColor;
  vec3 norm = normalize(fragNormal);
  vec3 lightDir = normalize(lightPos - inPos);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = lightColor * diff;

  vec4 objColor = texture(texSampler[sampler_idx], fragTexCoord) * vec4(fragColor, 1.0);
  outColor = vec4((amient + diffuse) * vec3(objColor), 1.0);
}

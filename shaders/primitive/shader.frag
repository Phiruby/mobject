#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(push_constant, std430) uniform pc {
  uint sampler_idx;
};
layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 inPos;
layout(location = 4) in vec4 fragPosLightSpace;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D texSampler[];
layout(set = 0, binding = 2) uniform sampler2DShadow shadowMap;

// taken from https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping
// float ShadowCalculation(vec4 fragPosLightSpace, float bias)
// {
//     // perform perspective divide
//     vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
//     // projCoords = projCoords * 0.5 + 0.5;
//     // float closestDepth = texture(shadowMap, projCoords.xy * 0.5 + 0.5).r;
//     float closestDepth = texture(shadowMap, projCoords.xy * 0.5 + 0.5).r;
//     // return closestDepth;
//     float currentDepth = projCoords.z;
//     float shadow = currentDepth - bias > closestDepth  ? 1.0 : 0.0;
//     return shadow;
// }

float ShadowCalculation(vec4 fragPosLightSpace, float bias)
{
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords.xy = projCoords.xy * 0.5 + 0.5;
    float visibility = texture(shadowMap, vec3(projCoords.xy, projCoords.z - bias));
    return 1.0 - visibility;
}

void main() {
  float ambientStrength = 0.1;
  vec3 lightColor = vec3(0.7, 0.7, 0.7);
  // TODO: light color separately stored here and scene.rs right now; unify
  vec3 lightPos = vec3(3.0, 3.0, 4.0);

  vec3 ambient = ambientStrength * lightColor;
  vec3 norm = normalize(fragNormal);
  if (!gl_FrontFacing) {
      norm = -norm;
  }
  vec3 lightDir = normalize(lightPos - inPos);
  float diff = max(dot(norm, lightDir), 0.0);
  vec3 diffuse = lightColor * diff;

  vec4 objColor = texture(texSampler[sampler_idx], fragTexCoord) * vec4(fragColor, 1.0);
  float shadow = ShadowCalculation(fragPosLightSpace, max(0.05 * (1.0 - dot(norm, lightDir)), 0.005));
  vec4 lighting = vec4((ambient + (1.0 - shadow) * diffuse), 1.0) * objColor;
  outColor = lighting;
}

#version 450
layout (vertices=9) out;
layout (location = 0) in vec3 fragColor[];
layout (location = 1) in vec2 texCoord[];

layout (location = 0) out vec2 textureCoord[];
layout (location = 1) out vec3 fragmentColors[];

in gl_PerVertex
{
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
    float gl_CullDistance[];
} gl_in[];

void main() {
  gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
  textureCoord[gl_InvocationID] = texCoord[gl_InvocationID];
  fragmentColors[gl_InvocationID] = fragColor[gl_InvocationID];

  if (gl_InvocationID == 0) {
      // https://docs.vulkan.org/spec/latest/chapters/tessellation.html#tessellation-triangle-tessellation
      gl_TessLevelInner[0] = 16.0;
      gl_TessLevelInner[1] = 16.0;
      // controls outer subdivisions
      gl_TessLevelOuter[0] = 16.0;
      gl_TessLevelOuter[1] = 16.0;
      gl_TessLevelOuter[2] = 16.0;
      gl_TessLevelOuter[3] = 16.0;
    }
}

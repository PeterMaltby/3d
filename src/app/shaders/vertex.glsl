#version 460 core

layout (location=0) out vec3 colour;

const vec2 pos[3] = vec2[3] (
  vec2(-0.6, -0.4),
  vec2(0.6, -0.4),
  vec2(-0.0, 0.6),
);

const vec3 col[3] = vec3[3] (
  vec3(-0.6, -0.4),
  vec3(0.6, -0.4),
  vec3(-0.0, 0.6),
);

void main() {
  gl_Position = vec4(pos[gl_VertexID], 0.0, 1.0);
  colour = col[gl_VertexID];
}

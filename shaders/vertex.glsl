#version 460 core

layout (std140, binding = 0) uniform perFrameData {
  uniform mat4 translation_matrix;
  uniform int is_wire_frame;
};

layout (location=0) out vec3 color;

const vec3 position[8] = vec3[8] (
  vec3(-1.0, -1.0, 1.0), 
  vec3(1.0, -1.0, 1.0),
  vec3(1.0, 1.0, 1.0), 
  vec3(-1.0, 1.0, 1.0),

  vec3(-1.0, -1.0, -1.0),
  vec3(1.0, -1.0, -1.0),
  vec3(1.0, 1.0, -1.0),
  vec3(-1.0, 1.0, -1.0)
);

const vec3 colour[8] = vec3[8] (
  vec3(-1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0),
  vec3(0.0, 0.0, 1.0), vec3(1.0, 1.0, 0.0),
  vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0),
  vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)
);

const int indices[36] = int[36](
  //front
  0,1,2,2,3,0,
  //right
  1,5,6,6,2,1,
  //back
  7,6,5,5,4,7,
  //left
  4,0,3,3,7,4,
  //bottom
  4,5,1,1,0,4,
  //top
  3,2,6,6,7,3
);

void main() {
  int idx = indices[gl_VertexID];
  gl_Position = translation_matrix * vec4(position[idx], 1.0);
  color = is_wire_frame > 0 ? vec3(0.0) : colour[idx];
}

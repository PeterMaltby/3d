#version 460 core

layout (location=0) in vec3 colour;
layout (location=0) out vec4 out_FragColour;
void main() {
  out_FragColour = vec4(colour, 1.0);
};


#version 460 core
layout (location=0) in vec2 uv;
layout (location=0) out vec4 out_FragColor;
uniform sampler2D texture0;

void main() {
  out_FragColor = texture(texture0, uv);
}

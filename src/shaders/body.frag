#version 430 core

out vec4 color;

uniform vec4 uni_color;

void main()
{
    color = uni_color;
    // color = vec4(0.0, 1.0, 0.0, 1.0);
}
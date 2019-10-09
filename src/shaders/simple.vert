#version 430 core

in vec3 position;

uniform mat4 trans;
uniform vec4 displacement;

void main()
{
    gl_Position = trans * vec4(position, 1.0f) + displacement;
}
#version 430 core

in vec3 position;
in vec4 in_color;

out vec4 frag_color;

uniform mat4 trans;

void main()
{
    gl_Position = trans * vec4(position, 1.0f);

    frag_color = in_color;
}
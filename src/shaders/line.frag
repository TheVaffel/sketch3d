#version 430 core

in vec4 frag_color;

out vec4 color;

void main()
{
    color = frag_color; // uni_color;
}
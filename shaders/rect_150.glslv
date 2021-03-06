// shaders/rect_150.glslv

#version 150 core

in vec4 a_Pos;
in vec3 a_Color;

uniform mat4 u_Model;
uniform mat4 u_View;
uniform mat4 u_Proj;

out vec4 v_Color;

void main() {
    v_Color = vec4(a_Color, 1.0);
    gl_Position = u_Proj*u_View*u_Model*a_Pos;
}
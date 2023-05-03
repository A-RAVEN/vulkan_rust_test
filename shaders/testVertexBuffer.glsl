#version 450
#extension GL_ARB_separate_shader_objects : enable

#ifdef SHADER_FREQUENCY_VERTEX
#define VERTEX_INPUT(locationId, dataType, name) layout(location = locationId) in dataType name;
#define VERTEX_TO_FRAGMENT(locationId, dataType, name) layout(location = locationId) out dataType name;
#define FRAGMENT_OUTPUT(locationId, dataType, name)
#endif
#ifdef SHADER_FREQUENCY_FRAGMENT
#define VERTEX_INPUT(locationId, dataType, name)
#define VERTEX_TO_FRAGMENT(locationId, dataType, name) layout(location = locationId) in dataType name;
#define FRAGMENT_OUTPUT(locationId, dataType, name) layout(location = locationId) out dataType name;
#endif

VERTEX_INPUT(0, vec2, input_Position)
VERTEX_INPUT(1, vec3, input_Color)
VERTEX_TO_FRAGMENT(0, vec3, v2f_Color)
FRAGMENT_OUTPUT(0, vec4, outColor)

#if SHADER_FREQUENCY_VERTEX
out gl_PerVertex {
    vec4 gl_Position;
};
void main()
{
    gl_Position = vec4(input_Position, 0, 1);
    v2f_Color = input_Color;
}
#endif//SHADER_FREQUENCY_VERTEX
#if SHADER_FREQUENCY_FRAGMENT
void main()
{
    outColor = vec4(v2f_Color, 1.0);
}
#endif//SHADER_FREQUENCY_FRAGMENT

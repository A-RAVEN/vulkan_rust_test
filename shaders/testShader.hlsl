
struct VertexToFragment
{
    float4 position : SV_POSITION;
    [[vk::location(0)]] float3 color : COLOR;
};

struct VertexInput
{
    [[vk::location(0)]] float2 position : POSITION;
    [[vk::location(1)]] float3 color : COLOR; 
};

#if SHADER_FREQUENCY_VERTEX
VertexToFragment vert(in VertexInput input)
{
    VertexToFragment result = (VertexToFragment)0;
    result.position = float4(input.position, 0, 1);
    result.color = input.color;
    return result;
}
#endif

#if SHADER_FREQUENCY_FRAGMENT
[[vk::location(0)]] float4 frag(in VertexToFragment input) : SV_TARGET0
{
    return float4(input.color, 1.0);
}
#endif
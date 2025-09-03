// Vertex shader for Blish HUD overlay
// Generates a full-screen triangle without vertex buffer

struct VS_OUTPUT {
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD0;
};

VS_OUTPUT main(uint vertexID : SV_VertexID) {
    VS_OUTPUT output;
    
    // Generate full-screen triangle coordinates
    // This creates a triangle that covers the entire screen
    float2 texcoord = float2((vertexID << 1) & 2, vertexID & 2);
    output.position = float4(texcoord * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
    output.texcoord = texcoord;
    
    return output;
}
// Vertex shader for Blish HUD overlay with proper scaling support
// Uses vertex buffer and constant buffer for flexible positioning

cbuffer Constants : register(b0) {
    float2 TextureScale;
    float2 TextureOffset;
};

struct VS_INPUT {
    float2 position : POSITION;
    float2 texcoord : TEXCOORD0;
};

struct VS_OUTPUT {
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD0;
};

VS_OUTPUT main(VS_INPUT input) {
    VS_OUTPUT output;
    
    // Pass through position (already in clip space)
    output.position = float4(input.position, 0.0, 1.0);
    
    // Apply scaling to texture coordinates to handle size mismatches
    output.texcoord = input.texcoord * TextureScale + TextureOffset;
    
    return output;
}
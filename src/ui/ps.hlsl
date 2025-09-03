// Pixel shader for Blish HUD overlay
// Samples the overlay texture and outputs it with proper alpha blending

Texture2D overlayTexture : register(t0);
SamplerState overlaysampler : register(s0);

struct PS_INPUT {
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD0;
};

float4 main(PS_INPUT input) : SV_TARGET {
    return overlayTexture.Sample(overlaysampler, input.texcoord);
}
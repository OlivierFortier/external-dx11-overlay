use std::{
    sync::{Mutex, atomic::Ordering},
    time::Instant,
};

use windows::{
    Win32::{
        Foundation::{BOOL, HANDLE},
        Graphics::{
            Direct3D::{D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D11_SRV_DIMENSION_TEXTURE2D},
            Direct3D11::{
                D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
                D3D11_BLEND_SRC_ALPHA, D3D11_BLEND_ZERO, D3D11_COLOR_WRITE_ENABLE_ALL,
                D3D11_COMPARISON_NEVER, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_FLOAT32_MAX,
                D3D11_SAMPLER_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_TEXTURE_ADDRESS_CLAMP,
                D3D11_VIEWPORT, ID3D11BlendState, ID3D11Device, ID3D11DeviceContext,
                ID3D11PixelShader, ID3D11RenderTargetView, ID3D11SamplerState,
                ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader,
                D3D11_TEXTURE2D_DESC,
            },
            Dxgi::{Common::DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SWAP_CHAIN_DESC, IDXGISwapChain},
        },
    },
    core::{Error, HRESULT},
};

use crate::{
    debug::{
        DEBUG_FEATURES,
        statistics::{self, send_statistic},
    },
    hooks::present_hook,
    ui::{MMF_DATA, mmf::cleanup_shutdown},
};

use super::OVERLAY_STATE;

//Ultra basic shader.
//Have to be compiled on windows with fxc.
static VS_OVERLAY: &[u8] = include_bytes!("vs.cso");
static PS_OVERLAY: &[u8] = include_bytes!("ps.cso");

//Contains DirectX related stuff that can be reused over many frames.
pub struct OverlayState {
    pub backbuffer_width: u32,
    pub backbuffer_height: u32,
    pub source_width: u32,
    pub source_height: u32,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    overlay_textures: [Option<ID3D11Texture2D>; 2],
    shader_resource_views: [Option<ID3D11ShaderResourceView>; 2],
    render_target_view: Option<ID3D11RenderTargetView>,
    blend_state: ID3D11BlendState,
    sampler_state: ID3D11SamplerState,
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    viewport: D3D11_VIEWPORT,
    blend_factor: [f32; 4],
    source_addrs: [u64; 2],
}

impl OverlayState {
    pub fn resize(&mut self, swapchain: &IDXGISwapChain) {
        // Query the actual backbuffer texture size to avoid stale DXGI swapchain desc in windowed modes
        let mut bb_w: u32 = 0;
        let mut bb_h: u32 = 0;
        unsafe {
            if let Ok(backbuffer) = swapchain.GetBuffer::<ID3D11Texture2D>(0) {
                let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
                backbuffer.GetDesc(&mut bb_desc);
                bb_w = bb_desc.Width;
                bb_h = bb_desc.Height;
            }
        }
        if bb_w == 0 || bb_h == 0 {
            // Fallback to swapchain desc if needed
            let mut desc = DXGI_SWAP_CHAIN_DESC::default();
            unsafe { swapchain.GetDesc(&mut desc).ok(); }
            bb_w = desc.BufferDesc.Width.max(1);
            bb_h = desc.BufferDesc.Height.max(1);
        }

        self.viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: bb_w as f32,
            Height: bb_h as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        self.backbuffer_width = bb_w;
        self.backbuffer_height = bb_h;

        self.render_target_view = create_render_target_view(swapchain, &self.device);
    }
    pub fn shutdown(&mut self) {
        self.overlay_textures = [None, None];
        self.shader_resource_views = [None, None];
        self.render_target_view.take();

        self.backbuffer_width = 0;
        self.backbuffer_height = 0;
        self.source_width = 0;
        self.source_height = 0;
        self.source_addrs = [0, 0];

        self.viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: 0.0,
            Height: 0.0,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        self.blend_factor = [0.0; 4];
    }
}

///This is our big present hook. Draws shared textures as an overlay.
pub fn detoured_present(swapchain: IDXGISwapChain, sync_interval: u32, flags: u32) -> HRESULT {
    let start = Instant::now();
    //Macro to make it less ugly to return early.
    macro_rules! return_present {
        () => {
            return present_hook.call(swapchain, sync_interval, flags)
        };
    }
    if !DEBUG_FEATURES.rendering_enabled.load(Ordering::Relaxed) {
        unsafe { return_present!() }
    }
    unsafe {
        if OVERLAY_STATE.get().is_none() {
            initialize_overlay_state(&swapchain);
        }

        //Check if we need to cache stuff over again
        let mut lock = OVERLAY_STATE.get().unwrap().lock().unwrap();
        let recreate = if let Some(state) = lock.as_ref() {
            if state.backbuffer_width == 0 || state.backbuffer_height == 0 {
                true
            } else {
                state.device.GetDeviceRemovedReason().is_err()
            }
        } else {
            true
        };
        if recreate {
            drop(lock);
            initialize_overlay_state(&swapchain);
            lock = OVERLAY_STATE.get().unwrap().lock().unwrap();
        }

        let mut state = lock.as_mut().unwrap();

        let mmfdata = MMF_DATA.get().unwrap().read().unwrap();

        //Which texture we should draw
        let texture_idx = mmfdata.index as usize;

        //Bad data, don't render that frame.
        if !mmfdata.is_blish_alive
            || mmfdata.width == 0
            || mmfdata.height == 0
            || mmfdata.addr1 == 0
            || mmfdata.addr2 == 0
        {
            return_present!();
        }

        // Ensure backbuffer RTV matches current swapchain backbuffer size
        let mut need_resize = false;
        unsafe {
            if let Ok(bb) = swapchain.GetBuffer::<ID3D11Texture2D>(0) {
                let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
                bb.GetDesc(&mut bb_desc);
                if state.backbuffer_width != bb_desc.Width || state.backbuffer_height != bb_desc.Height {
                    need_resize = true;
                }
            }
        }
        if need_resize || state.render_target_view.is_none() {
            state.resize(&swapchain);
        }

        // Ensure SRVs match current Blish HUD texture addresses/sizes
        if state.source_height != mmfdata.height
            || state.source_width != mmfdata.width
            || state.source_addrs != [mmfdata.addr1, mmfdata.addr2]
            || state.shader_resource_views[texture_idx].is_none()
        {
            if update_textures(&mut state, [mmfdata.addr1, mmfdata.addr2], mmfdata.width, mmfdata.height).is_err() {
                state.context.PSSetShaderResources(0, Some(&[None]));
                drop(mmfdata);
                drop(lock);
                cleanup_shutdown();
                return_present!();
            }
        }

        //Make sure SRV is valid
        if state.shader_resource_views[texture_idx].is_none() {
            return_present!();
        }

        let ctx = &state.context;

        // Backup critical pipeline state we modify
        let mut prev_rtv_arr: [Option<ID3D11RenderTargetView>; 1] = [None];
        let mut prev_dsv: Option<windows::Win32::Graphics::Direct3D11::ID3D11DepthStencilView> = None;
        let mut prev_blend: Option<ID3D11BlendState> = None;
        let mut prev_blend_factor: [f32; 4] = [0.0; 4];
        let mut prev_sample_mask: u32 = 0xffffffff;
        let mut prev_ps_srv0_arr: [Option<ID3D11ShaderResourceView>; 1] = [None];
        let mut prev_ps_sampler0_arr: [Option<ID3D11SamplerState>; 1] = [None];
        let mut prev_vs: Option<ID3D11VertexShader> = None;
        let mut prev_ps: Option<ID3D11PixelShader> = None;
        let mut prev_vs_num_classes: u32 = 0;
        let mut prev_ps_num_classes: u32 = 0;
        let mut prev_topology = D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
        // Viewport backup skipped to avoid API binding variance; most hosts reset per-frame

        {
            // Render targets
            ctx.OMGetRenderTargets(Some(&mut prev_rtv_arr), Some(&mut prev_dsv));
            // Blend state
            ctx.OMGetBlendState(Some(&mut prev_blend), Some(&mut prev_blend_factor), Some(&mut prev_sample_mask));
            // Shaders
            ctx.VSGetShader(&mut prev_vs, None, Some(&mut prev_vs_num_classes));
            ctx.PSGetShader(&mut prev_ps, None, Some(&mut prev_ps_num_classes));
            // Resources and samplers
            ctx.PSGetShaderResources(0, Some(&mut prev_ps_srv0_arr));
            ctx.PSGetSamplers(0, Some(&mut prev_ps_sampler0_arr));
            // Topology
            prev_topology = ctx.IAGetPrimitiveTopology();
            // Viewports not backed up
        }

        // Apply our pipeline state (avoid forcing viewport/state beyond what we restore)
        ctx.OMSetBlendState(&state.blend_state, Some(&state.blend_factor), 0xffffffff);
        ctx.OMSetRenderTargets(Some(&[state.render_target_view.clone()]), None);
        // Avoid overriding the game's viewport to prevent stretching or scaling issues
        ctx.VSSetShader(&state.vertex_shader, None);
        ctx.PSSetShader(&state.pixel_shader, None);
        ctx.PSSetShaderResources(
            0,
            Some(&[Some(
                state.shader_resource_views[texture_idx]
                    .as_ref()
                    .unwrap()
                    .clone(),
            )]),
        );
        ctx.PSSetSamplers(0, Some(&[Some(state.sampler_state.clone())]));
        ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        ctx.Draw(3, 0);

        // Restore previous pipeline state
        {
            let rtv_restore = [prev_rtv_arr[0].clone()];
            let dsv_restore = prev_dsv.as_ref();
            ctx.OMSetRenderTargets(Some(&rtv_restore), dsv_restore);
            ctx.OMSetBlendState(prev_blend.as_ref(), Some(&prev_blend_factor), prev_sample_mask);
            if let Some(vs) = prev_vs.as_ref() { ctx.VSSetShader(vs, None); } else { ctx.VSSetShader(None, None); }
            if let Some(ps) = prev_ps.as_ref() { ctx.PSSetShader(ps, None); } else { ctx.PSSetShader(None, None); }
            let srv_restore = [prev_ps_srv0_arr[0].clone()];
            ctx.PSSetShaderResources(0, Some(&srv_restore));
            let samp_restore = [prev_ps_sampler0_arr[0].clone()];
            ctx.PSSetSamplers(0, Some(&samp_restore));
            ctx.IASetPrimitiveTopology(prev_topology);
            // Viewport restore skipped
        }

        //Stats
        let frame_time_custom = start.elapsed().as_nanos() as u32;
        send_statistic(statistics::debug_stat::FRAME_TIME_CUSTOM, frame_time_custom);

        //Original present
        let result = present_hook.call(swapchain, sync_interval, flags);

        let frame_time_total = start.elapsed().as_nanos() as u32;
        send_statistic(statistics::debug_stat::FRAME_TIME_TOTAL, frame_time_total);
        send_statistic(
            statistics::debug_stat::FRAME_TIME_DIFF,
            frame_time_total - frame_time_custom,
        );
        return result;
    }
}

//Updates the texture from the shared resource.
fn update_textures(state: &mut OverlayState, texture_ptrs: [u64; 2], src_width: u32, src_height: u32) -> Result<(), ()> {
    state.overlay_textures = [None, None];
    state.shader_resource_views = [None, None];

    for i in 0..2 {
        unsafe {
            if let Err(e) = state.device.OpenSharedResource(
                HANDLE(texture_ptrs[i] as isize),
                &mut state.overlay_textures[i] as *mut _,
            ) {
                log::error!("Failed to open shared resource: {}", e.to_string());
                return Err(());
            }
        };
        let tex = state.overlay_textures[i].as_ref().unwrap();
        let mut srv: Option<ID3D11ShaderResourceView> = None;

        let desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
            Anonymous: windows::Win32::Graphics::Direct3D11::D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
                Texture2D: windows::Win32::Graphics::Direct3D11::D3D11_TEX2D_SRV {
                    MostDetailedMip: 0,
                    MipLevels: 1,
                },
            },
        };

        unsafe {
            if let Err(e) = state
                .device
                .CreateShaderResourceView(tex, Some(&desc), Some(&mut srv))
            {
                log::error!("Failed to create shader resource view: {}", e.to_string());
                return Err(());
            }
        }
        state.shader_resource_views[i] = srv;
    }
    // Track current source texture properties to detect changes reliably
    state.source_width = src_width;
    state.source_height = src_height;
    state.source_addrs = texture_ptrs;
    Ok(())
}

fn get_device_and_context(
    swapchain: &IDXGISwapChain,
) -> Result<(ID3D11Device, ID3D11DeviceContext), ()> {
    unsafe {
        if let Ok(device) = swapchain.GetDevice::<ID3D11Device>() {
            if let Ok(ctx) = (device).GetImmediateContext() {
                return Ok((device, ctx));
            }
        }
    }
    Err(())
}

fn initialize_overlay_state(swapchain: &IDXGISwapChain) {
    let (device, context) =
        get_device_and_context(swapchain).expect("Could not get device and context from swapchain");
    let state = OverlayState {
        backbuffer_width: 0,
        backbuffer_height: 0,
        source_width: 0,
        source_height: 0,
        device: device.clone(),
        context: context.clone(),
        blend_state: create_blend_state(&device).unwrap(),
        sampler_state: create_sampler_state(&device).unwrap(),
        vertex_shader: create_vertex_shader(&device).unwrap(),
        pixel_shader: create_pixel_shader(&device).unwrap(),
        overlay_textures: [None, None],
        shader_resource_views: [None, None],
        viewport: D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: 0 as f32,
            Height: 0 as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        },
        render_target_view: create_render_target_view(swapchain, &device),
        blend_factor: [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        source_addrs: [0, 0],
    };
    let overlay_state = OVERLAY_STATE.get_or_init(|| Mutex::new(None));
    if let Ok(mut lock) = overlay_state.lock() {
        *lock = Some(state);
    } else {
        log::error!("Poisoned OVERLAY_STATE mutex.");
    }
}

pub fn create_render_target_view(
    swapchain: &IDXGISwapChain,
    device: &ID3D11Device,
) -> Option<ID3D11RenderTargetView> {
    unsafe {
        let backbuffer: Result<ID3D11Texture2D, _> = swapchain.GetBuffer(0);
        if let Ok(buffer) = backbuffer {
            let mut rtv: Option<ID3D11RenderTargetView> = None;
            if device
                .CreateRenderTargetView(&buffer, None, Some(&mut rtv))
                .is_ok()
            {
                return rtv;
            }
        }
    }
    None
}

///Creates the vertex shader to be used to display the overlay. Will be reused forever.
pub fn create_vertex_shader(device: &ID3D11Device) -> Result<ID3D11VertexShader, Error> {
    let mut vs: Option<ID3D11VertexShader> = None;
    unsafe {
        device.CreateVertexShader(VS_OVERLAY, None, Some(&mut vs))?;
    }
    Ok(vs.unwrap())
}

///Creates the pixel shader to be used to display the overlay. Will be reused forever.
pub fn create_pixel_shader(device: &ID3D11Device) -> Result<ID3D11PixelShader, Error> {
    let mut ps: Option<ID3D11PixelShader> = None;
    unsafe {
        device.CreatePixelShader(PS_OVERLAY, None, Some(&mut ps))?;
    }
    Ok(ps.unwrap())
}

///Creates the SamplerState to be used to display the overlay. Will be reused forever.
pub fn create_sampler_state(device: &ID3D11Device) -> Result<ID3D11SamplerState, Error> {
    let sampler_desc = D3D11_SAMPLER_DESC {
        Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
        ComparisonFunc: D3D11_COMPARISON_NEVER,
        MinLOD: 0.0,
        MaxLOD: D3D11_FLOAT32_MAX,
        ..Default::default()
    };

    let mut sampler: Option<ID3D11SamplerState> = None;
    unsafe {
        device.CreateSamplerState(&sampler_desc, Some(&mut sampler))?;
    }

    Ok(sampler.unwrap())
}

///Creates the BlendState to be used to display the overlay. Will be reused forever.
///Required for transparency / alpha blending
pub fn create_blend_state(device: &ID3D11Device) -> Result<ID3D11BlendState, Error> {
    let mut blend_desc = D3D11_BLEND_DESC::default();

    blend_desc.RenderTarget[0].BlendEnable = BOOL(1);
    blend_desc.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
    blend_desc.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
    blend_desc.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
    blend_desc.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ONE;
    blend_desc.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_ZERO;
    blend_desc.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
    blend_desc.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;

    let mut blend_state: Option<ID3D11BlendState> = None;
    unsafe {
        device.CreateBlendState(&blend_desc, Some(&mut blend_state))?;
    }

    Ok(blend_state.unwrap())
}

## Overview

This project is a Windows-only Rust cdylib that injects into a DirectX 11 application and renders an external overlay by hooking IDXGISwapChain::Present. It was built to host Blish HUD inside the game via shared textures and can also integrate with the Nexus framework (feature flag).

### Runtime Modes
- Standalone (default features): Sets up a Present hook and renders shared textures directly.
- Nexus integration (feature `nexus`): Intended to skip the Present hook and rely on Nexus/ImGui for UI. Note: in the current code, several `#[cfg(not(feature = "nexus"))]` guards are commented out, so standalone logic still runs under `nexus`. See the Nexus document for details.

## High-level Data Flow
1) Blish HUD (or another source) publishes two shared DX11 textures and a header MMF `BlishHUD_Header` containing: width, height, active index, and two shared handles.
2) The overlay DLL hooks Present. On each frame it:
   - Ensures device/context/RTV cached in `OverlayState` are valid
   - Opens the current shared texture via `ID3D11Device::OpenSharedResource`
   - Binds SRV + sampler and draws a full-screen triangle using precompiled shaders
   - Emits simple timing statistics
3) Input: A custom WndProc intercepts mouse/key events and sends minimal UDP packets for consumers.
4) Logging: Uses `fern` to log to stdout and to `addons/LOADER_public/logs/overlay-*.log`. Logs are also mirrored to a simple debug overlay buffer.

## Key Components

### Entry and Lifecycle (`src/lib.rs`)
- DllMain → `attach` on process attach, `detatch` on detach
- Initializes logging and panic hook
- Spawns threads:
  - MMF thread (`ui::mmf::start_mmf_thread`) to read header data frequently into `MMF_DATA`
  - Statistics server (`debug::statistics::start_statistics_server`)
- Finds DX11 Present address via `AddressFinder::find_addr_present` and installs detour with `retour::static_detour!`
- Sets up input routing: mouse thread + custom WndProc

### Hooking and Rendering (`src/ui/rendering.rs`)
- `detoured_present(swapchain, ...)` is the detour target returned via `ui::get_detoured_present()`
- Maintains reusable `OverlayState`:
  - Cached device, immediate context, RTV, SRVs, shaders, sampler, blend state, viewport
  - Rebuilds resources on resize or device removal
- Pulls `MMF_DATA` (width, height, index, two shared texture handles)
- Opens shared resource handles as `ID3D11Texture2D`, creates SRVs, sets pipeline state, and draws
- Measures custom and total frame time and sends statistics

### Shared Memory and Liveness (`src/ui/mmf.rs`)
- Background thread updates a `RwLock<MMFData>` from header MMF `BlishHUD_Header`
- Fields: `width`, `height`, `index`, `addr1`, `addr2`, `is_blish_alive`
- Liveness is probed via a named mutex `Global\\blish_isalive_mutex`
- On shutdown/crash, cleans mapped views/handles and signals `OverlayState` to reset

### Input and Keybinds (`src/controls.rs`, `src/keybinds.rs`)
- Subclasses the host window WndProc with `SetWindowLongPtrW`
- Handles:
  - `WM_MOUSEMOVE` → sends a small UDP packet with id/x/y to `127.0.0.1:49152`
  - `WM_KEYDOWN` → looks up configured keybinds
- Focus management (foreground, capture, focus)
- Keybinds are loaded from `addons/LOADER_public/keybinds.conf` (created with defaults if missing):
  - Toggles rendering/processing/debug overlay and overlay mode, dumps debug, restarts Blish

### Debug Overlay and Statistics (`src/debug/*`)
- `debug_overlay.rs`: CPU-side RGBA buffer (600x180) with a simple rasterized text renderer (embedded `segoeui.ttf`).
  - Two modes: log view and simple statistics view
  - Buffer can be refreshed on-demand; a copy-to-frame API exists but is not wired in `detoured_present`
- `statistics.rs`: channel-based aggregator for frame timing values; refreshes overlay when enabled

### Address Discovery (`src/address_finder.rs`)
- Creates a dummy window + DXGI swap chain + device, then extracts the Present function address from the vtable
- Also includes a KMP-based pattern search utility (not used by default)

### Utilities and Globals (`src/utils.rs`, `src/globals.rs`, `src/hooks.rs`)
- `utils.rs`: process base+size, host HWND discovery, window enumeration helpers, safe pointer read
- `globals.rs`: global original WndProc and UDP target
- `hooks.rs`: declares the `present_hook` detour type

## Logging, Errors, and Safety
- Logging: `fern` + `log` with timestamps; errors include file/line for quick diagnosis; mirrored to debug overlay
- Panic hook logs panics with location
- Many operations are `unsafe` and defensive early-returns are used in the detour to preserve game stability

## Performance Considerations
- Render path uses a single full-screen triangle with alpha blending and linear sampling
- Shared resource SRVs are recreated on resize; otherwise resources are reused
- MMF reads are minimized under the write lock; thread sleeps ~20ms

## Notable Findings
- The code comments indicate conditional compilation for standalone vs Nexus, but several `#[cfg(not(feature = "nexus"))]` guards are commented out. As-is, Present hooking and other standalone setup still execute with the `nexus` feature enabled. Adjust the `cfg` gates to avoid double ownership of rendering/input paths under Nexus.
- The debug overlay buffer is kept in sync but is not currently composited in `detoured_present`.



## Crate Layout

Top-level modules and their roles.

### src/lib.rs
- Entry point (DllMain) and addon lifecycle:
  - Logging setup (fern), panic hook
  - Threads: MMF reader, statistics
  - Present hook installation via `retour`
  - Input setup (mouse thread + WndProc subclass)

### src/address_finder.rs
- `AddressFinder::find_addr_present`: creates a temporary DXGI swap chain to read Present vtable address
- Optional KMP pattern scan utility

### src/hooks.rs
- Declares the `present_hook` detour type

### src/ui
- `mod.rs`: globals for `MMF_DATA` and `OVERLAY_STATE`, exposes `get_detoured_present`
- `mmf.rs`: background thread reading header MMF `BlishHUD_Header` and liveness mutex; exposes `cleanup_shutdown`
- `rendering.rs`: `OverlayState` (device/context/RTV/SRVs/shaders/sampler/viewport). `detoured_present` draws the overlay
- Shaders: `vs.cso`, `ps.cso` precompiled HLSL blobs

### src/controls.rs
- WndProc hook (mouse/key/focus). Mouse packets are sent via UDP to `127.0.0.1:49152`

### src/keybinds.rs
- Loads `addons/LOADER_public/keybinds.conf`, dumps defaults if missing
- Maps key combinations (Ctrl/Alt/Shift + char) to actions:
  - Render/processing toggles
  - Debug overlay toggles and mode switches
  - Debug dump and Blish restart

### src/debug
- `debug_overlay.rs`: software overlay buffer and basic font rendering (embedded `segoeui.ttf`)
- `statistics.rs`: channel-based collector for per-frame timings with optional overlay refresh
- `mod.rs`: `DEBUG_FEATURES` flags and helpers (`dump_debug_data`, `restart_blish`, process killer)

### src/utils.rs
- Process base/size, find main window, enumerate windows, read pointer

### src/globals.rs
- `ORIGINAL_WNDPROC`, `LIVE_MUTEX` (unused in current code), `UDPADDR`

### src/nexus_addon (feature = "nexus")
- `init.rs`: Nexus entry (`nexus_load`/`nexus_unload`), texture loading, quick access, keybind, calls into `attach`
- `manager.rs`: single-executable manager with persistence (`exes.txt`) and process control
- `ui.rs`: ImGui window to configure and manage the executable
- `mod.rs`: error types and re-exports

## Cross-cutting behaviors
- Logging to file and debug overlay; panic hook for diagnostics
- Statistics emitted each frame (custom render, total, diff)
- Graceful device lost / resize handling in rendering path



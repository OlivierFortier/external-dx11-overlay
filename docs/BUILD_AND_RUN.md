## Prerequisites
- Rust toolchain (nightly not strictly required for current code; project uses edition 2024)
- Windows 10/11 with DX11 runtime
- For Nexus builds: Nexus SDK (via `nexus-rs` git dependency), and the target host (e.g., GW2 via loader)

Dependencies declared in `Cargo.toml`:
- `windows` crate with DXGI/D3D11 and related Win32 features
- `retour` for function detouring
- `fern`, `log`, `chrono` for logging
- `fontdue` for CPU text rasterization (debug overlay)
- `rfd` for file dialogs (Nexus UI)
- `nexus` (git) for Nexus integration

## Build

### Standalone
```bash
cargo build --release
```
Artifacts (MSVC target): `target/x86_64-pc-windows-msvc/release/external_dx11_overlay.dll`

Release profile is tuned for small, stripped binaries with LTO.

### Nexus
```bash
cargo build --features nexus --release
```

Exports a Nexus addon via `nexus::export!` in `src/lib.rs`.

## Deploy and Run (Standalone)
1) Place the built DLL alongside your loader or use any LoadLibraryW-based injector to inject into the target DX11 process.
2) Ensure Blish HUD (or your texture producer) runs and publishes the header MMF `BlishHUD_Header` and two shared textures.
3) Ensure shared textures are supported (e.g., DXVK >= 1.10.1 on Proton per project README/guide).
4) On first run, keybinds config is written to `addons/LOADER_public/keybinds.conf`.

Keybind defaults (examples):
- Ctrl+Alt+B: toggle rendering
- Ctrl+Alt+N: toggle processing
- Ctrl+Alt+D: toggle debug overlay
- Ctrl+Alt+Shift+1/2: overlay modes (log/statistics)
- Ctrl+Alt+P: dump debug data
- Ctrl+Alt+O: restart Blish HUD (uses default path or Nexus-configured path when feature enabled)

Logs: `addons/LOADER_public/logs/overlay-*.log`. Panics and errors include file/line.

## Deploy and Run (Nexus)
1) Build with `--features nexus`.
2) Load as a Nexus addon. The addon will:
   - Register quick access, keybind, and UI render callback
   - Provide a window to set and launch the Blish HUD executable (persisted in `exes.txt` under the addon dir)
3) By default, the code also calls `attach()` which installs the Present hook; consider restoring `cfg(not(feature = "nexus"))` guards to avoid duplicate rendering ownership.

## Troubleshooting
- See `Simple-User-Guide.md` and `Troubleshooting-Guide.md` for Linux/Proton instructions and common issues
- Verify MMF header values are non-zero and liveness mutex is detected
- If Present hook fails, confirm `AddressFinder::find_addr_present()` returns a valid address
- For device removal or resize issues, ensure the render target and SRVs are recreated (handled by `OverlayState::resize`)



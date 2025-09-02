## Nexus Integration

This crate optionally integrates with the `nexus` framework via the `nexus` Cargo feature. When enabled, the addon exposes a Nexus entrypoint and provides a minimal ImGui UI and executable management.

### What Nexus Provides Here
- Texture loading for addon icons
- Quick-access shortcut registration
- Keybind registration and callbacks
- ImGui render callback registration

### Code Paths
- `src/nexus_addon/mod.rs`: module root and error types; re-exports `nexus_load`/`nexus_unload`
- `src/nexus_addon/init.rs`:
  - `nexus_load()`: orchestrates initialization (addon dir, manager, textures, quick access, keybinds, UI render), then calls `crate::attach()` using the HINSTANCE obtained from `GetModuleHandleW`
  - `nexus_unload()`: stops running exe via manager, then calls `crate::detatch()`
- `src/nexus_addon/manager.rs`: single-executable manager with persistence in `exes.txt` (first line: path; second line: `launch_on_startup` bool). Provides `launch_exe`, `stop_exe`, `clear_exe`, `save_settings`, `cleanup_finished_process`
- `src/nexus_addon/ui.rs`: ImGui-based window with:
  - Browse to set executable path (using `rfd`)
  - Launch on startup checkbox
  - Launch/Stop/Remove actions
  - `toggle_window()` bound to a keybind (`ALT+SHIFT+1`)

### Present Hook and Rendering Ownership
The intention is for Nexus builds to avoid installing the Present hook and to let Nexus manage rendering/ImGui. However, several `#[cfg(not(feature = "nexus"))]` guards in `src/lib.rs` are commented out. As a result, Present hooking and input/keybind setup still occur even when `nexus` is enabled.

If you want strict separation:
1) Reinstate the `#[cfg(not(feature = "nexus"))]` guards around:
   - MMF thread if not required under Nexus
   - Present address discovery and hook initialization
   - Statistics server and standalone keybinds
   - Mouse input thread and WndProc replacement
2) Provide Nexus-side equivalents or no-ops where appropriate

### Build
Enable feature:

```bash
cargo build --features nexus --release
```

The crate exports the Nexus addon metadata in `src/lib.rs` under `#[cfg(feature = "nexus")]` using `nexus::export!`.



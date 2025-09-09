# Nexus Integration (Alternative Loader)

## Installation Steps with Nexus Addon Loader & Manager
This addon supports [Nexus Addon Loader & Manager](https://raidcore.gg/Nexus).

For simple installation steps, please refer to the [Nexus-specific User Guide](./Simple-User-Guide-Nexus.md).

For troubleshooting, please refer to the [Nexus-specific Troubleshooting Guide](./Troubleshooting-Guide-Nexus.md).

## Dev Notes

This crate optionally integrates with the `nexus` framework via the `nexus-rs` Cargo feature. When enabled, the addon exposes a Nexus entrypoint and provides a minimal ImGui UI.

### What Nexus Provides Here

- Addon discovery through the Nexus addon library (optionnal)
- Texture loading for addon icons
- Quick-access shortcut registration
- Keybind registration and callbacks
- ImGui render callback registration
- Addon metadata (name, version, author, description, etc)
- Addon lifecycle management (load/unload)
- Error reporting to the Nexus UI
- Automatic updates (optionnal)

The crate exports the Nexus addon metadata in `src/lib.rs` under `#[cfg(feature = "nexus")]` using `nexus::export!`.

### Modules

- `src/nexus_addon/mod.rs`: module root and error types; re-exports `nexus_load`/`nexus_unload`
- `src/nexus_addon/init.rs`:
  - `nexus_load()`: orchestrates initialization (addon dir, textures, quick access, keybinds, UI render), then calls `crate::attach()`
  - `nexus_unload()`: calls `crate::detatch()`
- `src/nexus_addon/ui.rs`: ImGui-based UI

### Build

```bash
cargo build --features nexus --release
```

## Known issues

- Unloading the addon while the game is running causes a crash and potentially a memory leak. This is normal as it is not properly implemented yet.

### Windows-specific: :
- Doesn't really work with windowed mode. Often creates a black screen with the nexus UI flickering.
- Weird scaling issue on fullscreen-windowed mode. The game and overlay works but sometimes the game can get stretched outside the screen and the event listener areas (where you click) are offset from where the actual UI is.
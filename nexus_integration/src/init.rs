/*!
# Nexus Addon Initialization Module

Handles initialization, resource loading, and cleanup for the Nexus addon.
Provides the main entry points for the addon lifecycle and orchestrates setup of UI, keybinds, quick access, and textures.

*/

use crate::nexus_addon::{NexusError, Result, ui};
use nexus::{
    keybind::register_keybind_with_string,
    keybind_handler,
    paths::get_addon_dir,
    quick_access::add_quick_access,
    texture::{RawTextureReceiveCallback, load_texture_from_memory},
    texture_receive,
};
use windows::Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleHandleW};

/// Returns the HMODULE and casts it into HINSTANCE
/// On modern systems, HMODULE is pretty much the same as HINSTANCE, and can be safely cast
fn get_hinstance() -> HINSTANCE {
    unsafe { GetModuleHandleW(None).unwrap().into() }
}

/// Nexus addon load function - entry point
pub fn nexus_load() {
    log::info!("Loading External DX11 overlay loader addon");

    if let Err(e) = initialize_nexus_addon() {
        log::error!("Failed to initialize nexus addon: {e}");
        return;
    }

    log::info!("External DX11 overlay loader addon loaded successfully");
}

fn initialize_nexus_addon() -> Result<()> {
    // Initialize the nexus menus and options
    // Create the addon dir if it doesn't exist
    use std::fs;

    let addon_dir = get_addon_dir("LOADER_public").ok_or_else(|| {
        NexusError::ManagerInitialization("Failed to get addon directory".to_string())
    })?;

    fs::create_dir_all(&addon_dir).map_err(|e| {
        NexusError::ManagerInitialization(format!("Failed to create addon directory: {e}"))
    })?;

    load_addon_textures()?;
    setup_keybinds()?;
    setup_quick_access()?;
    ui::setup_main_window_rendering();

    // Start the main DLL functionality
    let hinstance = get_hinstance();
    log::info!("Loading via Nexus - HMODULE/HINSTANCE: {}", hinstance.0);
    crate::attach(hinstance);

    Ok(())
}

fn setup_keybinds() -> Result<()> {
    let main_window_keybind_handler = keybind_handler!(|id, is_release| {
        log::info!(
            "keybind {id} {}",
            if is_release { "released" } else { "pressed" }
        );
        if !is_release {
            ui::toggle_window();
        }
    });

    register_keybind_with_string(
        "BLISH_OVERLAY_LOADER_KEYBIND",
        main_window_keybind_handler,
        "ALT+SHIFT+1",
    )
    .revert_on_unload();

    log::info!("Keybinds setup successfully");
    Ok(())
}

fn load_addon_textures() -> Result<()> {
    let icon = include_bytes!("../images/64p_nexus_blish_loader.png");
    let icon_hover = include_bytes!("../images/64p_nexus_blish_loader.png");

    let receive_texture: RawTextureReceiveCallback = texture_receive!(|id, _texture| {
        log::info!("texture {id} loaded");
    });

    // Note: load_texture_from_memory doesn't return a Result, so we assume success
    load_texture_from_memory("BLISH_OVERLAY_LOADER_ICON", icon, Some(receive_texture));
    load_texture_from_memory(
        "BLISH_OVERLAY_LOADER_ICON_HOVER",
        icon_hover,
        Some(receive_texture),
    );

    log::info!("Addon textures loaded successfully");
    Ok(())
}

/// Sets up the quick access menu entry
fn setup_quick_access() -> Result<()> {
    // Note: add_quick_access doesn't return a Result, so we assume success
    add_quick_access(
        "BLISH_OVERLAY_LOADER_SHORTCUT",
        "BLISH_OVERLAY_LOADER_ICON",
        "BLISH_OVERLAY_LOADER_ICON_HOVER",
        "BLISH_OVERLAY_LOADER_KEYBIND",
        "External DX11 overlay loader",
    )
    .revert_on_unload();

    log::info!("Quick access menu setup successfully");
    Ok(())
}

/// Nexus addon unload function - handles cleanup of all nexus-specific functionality
pub fn nexus_unload() {
    log::info!("Unloading External DX11 overlay runner addon");
    // Perform main cleanup
    crate::detatch();

    log::info!("External DX11 overlay runner cleanup completed successfully");
}

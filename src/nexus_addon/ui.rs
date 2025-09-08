use nexus::{
    gui::register_render,
    imgui::{Ui, Window},
    render,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global state for tracking if the main window is open
pub static IS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Registers the main window rendering callback with nexus
pub fn setup_main_window_rendering() {
    let main_window = render!(|ui| {
        render_main_window(ui);
    });
    register_render(nexus::gui::RenderType::Render, main_window).revert_on_unload();
}

/// Renders the main DX11 Overlay Loader window
pub fn render_main_window(ui: &Ui) {
    let mut is_open = IS_WINDOW_OPEN.load(Ordering::Relaxed);
    if is_open {
        Window::new("External DX11 Overlay Runner")
            .opened(&mut is_open)
            .size([500.0, 400.0], nexus::imgui::Condition::FirstUseEver)
            .collapsible(false)
            .build(ui, || {
                render_window_content(ui);
            });
        IS_WINDOW_OPEN.store(is_open, Ordering::Relaxed);
    }
}

/// Renders the content inside the main window
fn render_window_content(ui: &Ui) {
    // render text to explain what the addon does. it should be brief.
    ui.text_wrapped("This addon allows you to run an external overlay inside the game, such as Blish HUD.");
    ui.separator();
    ui.text_wrapped("Run a compatible overlay executable to get started.");
    ui.separator();
    // explain that it is recommended to use my other addon called "Gw2 executable loader" to easily run external programs such as blish hud from within the game. and mention how it is particularly useful for steamos users in game mode.
    ui.text_wrapped("It is recommended to use the 'Gw2 Executable Loader' addon to easily run external programs such as Blish HUD from within the game. This is particularly useful for SteamOS users in Game Mode.");
    ui.separator();
}

/// Toggles the main window visibility
pub fn toggle_window() {
    IS_WINDOW_OPEN.store(!IS_WINDOW_OPEN.load(Ordering::Relaxed), Ordering::Relaxed);
}
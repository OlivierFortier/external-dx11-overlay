use std::{
    net::UdpSocket,
    slice::from_raw_parts,
    sync::{
        OnceLock,
        mpsc::{Sender, channel},
    },
};

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{
            CallWindowProcW, DefWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW, WM_KEYDOWN,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
        },
    },
};
#[cfg(not(feature = "nexus"))]
use windows::Win32::UI::WindowsAndMessaging::{WM_ACTIVATE, WM_ACTIVATEAPP, WM_KILLFOCUS, WM_SETFOCUS};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::Foundation::RECT;

use crate::{
    globals::{self, ORIGINAL_WNDPROC},
    keybinds::{KEYBINDS, get_current_keybind},
};

pub fn initialize_controls(hwnd: HWND) {
    unsafe {
        let old_wndproc = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as _);
        ORIGINAL_WNDPROC = Some(std::mem::transmute(old_wndproc));
    }
}

fn get_x_lparam(lparam: LPARAM) -> i32 {
    let lparam_u32 = lparam.0 as u32;
    let x = (lparam_u32 & 0xFFFF) as i16;
    x as i32
}

fn get_y_lparam(lparam: LPARAM) -> i32 {
    let lparam_u32 = lparam.0 as u32;
    let y = ((lparam_u32 >> 16) & 0xFFFF) as i16;
    y as i32
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct MouseInputPacket {
    id: u8,
    x: i32,
    y: i32,
}

//Unsafe way to send packets over to a thread.
//It's 100% safe as long as:
//- Thread is initialized before the first call
//- Sender is only used in wnd_proc
#[derive(Debug)]
struct StaticSender {
    sender: *const Sender<MouseInputPacket>,
}
unsafe impl Sync for StaticSender {}
unsafe impl Send for StaticSender {}
static MOUSE_SENDER: OnceLock<StaticSender> = OnceLock::new();

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    'local_handling: {
        match msg {
            //Mouse
            WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP => {
                let mut x = get_x_lparam(lparam);
                let mut y = get_y_lparam(lparam);

                // Scale to Blish HUD texture resolution if available to avoid click offset
                if let Some(mmf_arc) = crate::ui::MMF_DATA.get() {
                    if let Ok(mmf) = mmf_arc.read() {
                        let mmf_w = mmf.width as i32;
                        let mmf_h = mmf.height as i32;
                        if mmf_w > 0 && mmf_h > 0 {
                            unsafe {
                                let mut rc: RECT = RECT::default();
                                if GetClientRect(hwnd, &mut rc).is_ok() {
                                    let client_w = (rc.right - rc.left).max(1);
                                    let client_h = (rc.bottom - rc.top).max(1);
                                    x = (x as i64 * mmf_w as i64 / client_w as i64) as i32;
                                    y = (y as i64 * mmf_h as i64 / client_h as i64) as i32;
                                }
                            }
                        }
                    }
                }

                //let is_overlay_pixel = ui::is_overlay_pixel(x as u32, y as u32);

                //Mouse up/down are seemingly handled globally.
                //So we only need to pass MOUSEMOVE.
                let id = match msg {
                    WM_LBUTTONDOWN => 0,
                    WM_LBUTTONUP => 1,
                    WM_MOUSEMOVE => 2,
                    WM_RBUTTONDOWN => 3,
                    WM_RBUTTONUP => 4,
                    _ => break 'local_handling,
                };

                //Send packet to listening thread.
                let packet = MouseInputPacket { id, x, y };
                let sender = unsafe { &*MOUSE_SENDER.get().unwrap().sender };
                sender.send(packet).ok();
                /*if is_overlay_pixel && msg == WM_LBUTTONDOWN && msg == WM_RBUTTONDOWN {
                    return LRESULT(0);
                }*/
            }
            WM_KEYDOWN => {
                if let Some(map) = KEYBINDS.get() {
                    let combo = get_current_keybind(wparam.0 as u32);
                    if let Some(action) = map.get(&combo) {
                        action();
                        return LRESULT(0);
                    }
                }
            }
            // Focus management can interfere with Nexus; disable under Nexus builds
            #[cfg(not(feature = "nexus"))]
            WM_SETFOCUS => { grab_focus(hwnd) }
            #[cfg(not(feature = "nexus"))]
            WM_KILLFOCUS => { release_focus() }
            #[cfg(not(feature = "nexus"))]
            WM_ACTIVATEAPP | WM_ACTIVATE => { if wparam.0 != 0 { grab_focus(hwnd); } else { release_focus(); } }
            _ => {}
        }
    }
    unsafe {
        if let Some(original) = ORIGINAL_WNDPROC {
            CallWindowProcW(original, hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

#[cfg(not(feature = "nexus"))]
fn grab_focus(_hwnd: HWND) {}
#[cfg(not(feature = "nexus"))]
fn release_focus() {}

pub fn start_mouse_input_thread() {
    let (tx, rx) = channel::<MouseInputPacket>();

    MOUSE_SENDER
        .set(StaticSender {
            sender: Box::into_raw(Box::new(tx)),
        })
        .unwrap();

    std::thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
        for packet in rx {
            let data = unsafe {
                from_raw_parts(
                    &packet as *const MouseInputPacket as *const u8,
                    size_of::<MouseInputPacket>(),
                )
            };
            socket.send_to(data, globals::UDPADDR).ok();
        }
    });
}

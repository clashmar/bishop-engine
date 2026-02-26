//! Conversion utilities for window-related types.

use crate::window::CursorIcon;

/// Converts a bishop CursorIcon to a winit CursorIcon.
pub fn convert_cursor_icon(icon: CursorIcon) -> winit::window::CursorIcon {
    match icon {
        CursorIcon::Default => winit::window::CursorIcon::Default,
        CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
        CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
        CursorIcon::Move => winit::window::CursorIcon::Move,
        CursorIcon::Text => winit::window::CursorIcon::Text,
        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
        CursorIcon::EWResize => winit::window::CursorIcon::EwResize,
        CursorIcon::NSResize => winit::window::CursorIcon::NsResize,
        CursorIcon::NESWResize => winit::window::CursorIcon::NeswResize,
        CursorIcon::NWSEResize => winit::window::CursorIcon::NwseResize,
    }
}

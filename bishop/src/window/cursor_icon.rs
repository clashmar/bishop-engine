//! Cursor icon types for mouse cursor control.

/// Mouse cursor icon styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorIcon {
    /// Default system cursor (usually arrow).
    #[default]
    Default,
    /// Pointing hand cursor for clickable elements.
    Pointer,
    /// Crosshair cursor for precision selection.
    Crosshair,
    /// Move/drag cursor.
    Move,
    /// Text selection cursor (I-beam).
    Text,
    /// Not allowed/forbidden cursor.
    NotAllowed,
    /// East-West resize cursor.
    EWResize,
    /// North-South resize cursor.
    NSResize,
    /// Northeast-Southwest resize cursor.
    NESWResize,
    /// Northwest-Southeast resize cursor.
    NWSEResize,
}

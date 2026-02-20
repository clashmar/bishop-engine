/// Key codes matching common keyboard layouts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyCode {
    // Printable characters
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,

    // Navigation / editing
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,

    // Lock / system keys
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,

    // Keypad
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,

    // Modifiers
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,

    // Misc
    Menu,
    Back,
    Unknown,
}

/// Mouse button identifiers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input state abstraction for keyboard and mouse.
pub trait Input {
    /// Returns true if the key is currently held down.
    fn is_key_down(&self, key: KeyCode) -> bool;

    /// Returns true if the key was pressed this frame.
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// Returns true if the key was released this frame.
    fn is_key_released(&self, key: KeyCode) -> bool;

    /// Returns true if the mouse button is currently held down.
    fn is_mouse_button_down(&self, button: MouseButton) -> bool;

    /// Returns true if the mouse button was pressed this frame.
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// Returns true if the mouse button was released this frame.
    fn is_mouse_button_released(&self, button: MouseButton) -> bool;

    /// Returns the current mouse position in screen coordinates.
    fn mouse_position(&self) -> (f32, f32);

    /// Returns the mouse wheel scroll delta (horizontal, vertical).
    fn mouse_wheel(&self) -> (f32, f32);

    /// Returns characters typed this frame for text input.
    fn chars_pressed(&self) -> Vec<char>;
}

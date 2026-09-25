//! Input model: keyboard keys, mouse events and the cursor.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Key(Key),
    MouseMove(u32, u32),
    MouseButton(MouseButton, bool), // (button, pressed)
}

/// The on-screen cursor (pointer).
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            visible: true,
        }
    }
}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;

/// Represents the current state of the keyboard modifiers
///
/// Each flag represents a modifier and is set if this modifier is active.
#[derive(Debug, Default)]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// This is the "windows" key on PC and "command" key on Mac.
    pub logo: bool,
}

/// A value between 0 and 1 that some input has been activated
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LinearInputActivation(f32);

impl LinearInputActivation {
    pub fn try_from(val: f32) -> Result<Self, f32> {
        if 0.0 <= val && val <= 1.0 {
            Ok(Self(val))
        } else {
            Err(val)
        }
    }
    pub fn get(self) -> f32 {
        self.0
    }

    pub fn clamp(val: f32) -> Self {
        Self(val.clamp(0.0, 1.0))
    }
}

/// A 2d value with both components between -1 and 1 that some input has been activated
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VectorInputActivation(f32, f32);

impl VectorInputActivation {
    pub fn try_from(x: f32, y: f32) -> Result<Self, (f32, f32)> {
        if -1.0 <= x && x <= 1.0 && -1.0 <= y && y <= 1.0 {
            Ok(Self(x, y))
        } else {
            Err((x, y))
        }
    }
    pub fn get(self) -> (f32, f32) {
        (self.0, self.1)
    }

    pub fn clamp(x: f32, y: f32) -> VectorInputActivation {
        Self(x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0))
    }
}

/// A key on a keyboard
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

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

    /// The Escape key, next to F1.
    Escape,

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

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,

    Shift,
    Ctrl,
    Alt,
    Logo,
}

impl From<VirtualKeyCode> for KeyCode {
    fn from(value: VirtualKeyCode) -> Self {
        match value {
            VirtualKeyCode::Key1 => Self::Key1,
            VirtualKeyCode::Key2 => Self::Key2,
            VirtualKeyCode::Key3 => Self::Key3,
            VirtualKeyCode::Key4 => Self::Key4,
            VirtualKeyCode::Key5 => Self::Key5,
            VirtualKeyCode::Key6 => Self::Key6,
            VirtualKeyCode::Key7 => Self::Key7,
            VirtualKeyCode::Key8 => Self::Key8,
            VirtualKeyCode::Key9 => Self::Key9,
            VirtualKeyCode::Key0 => Self::Key0,
            VirtualKeyCode::A => Self::A,
            VirtualKeyCode::B => Self::B,
            VirtualKeyCode::C => Self::C,
            VirtualKeyCode::D => Self::D,
            VirtualKeyCode::E => Self::E,
            VirtualKeyCode::F => Self::F,
            VirtualKeyCode::G => Self::G,
            VirtualKeyCode::H => Self::H,
            VirtualKeyCode::I => Self::I,
            VirtualKeyCode::J => Self::J,
            VirtualKeyCode::K => Self::K,
            VirtualKeyCode::L => Self::L,
            VirtualKeyCode::M => Self::M,
            VirtualKeyCode::N => Self::N,
            VirtualKeyCode::O => Self::O,
            VirtualKeyCode::P => Self::P,
            VirtualKeyCode::Q => Self::Q,
            VirtualKeyCode::R => Self::R,
            VirtualKeyCode::S => Self::S,
            VirtualKeyCode::T => Self::T,
            VirtualKeyCode::U => Self::U,
            VirtualKeyCode::V => Self::V,
            VirtualKeyCode::W => Self::W,
            VirtualKeyCode::X => Self::X,
            VirtualKeyCode::Y => Self::Y,
            VirtualKeyCode::Z => Self::Z,
            VirtualKeyCode::Escape => Self::Escape,
            VirtualKeyCode::F1 => Self::F1,
            VirtualKeyCode::F2 => Self::F2,
            VirtualKeyCode::F3 => Self::F3,
            VirtualKeyCode::F4 => Self::F4,
            VirtualKeyCode::F5 => Self::F5,
            VirtualKeyCode::F6 => Self::F6,
            VirtualKeyCode::F7 => Self::F7,
            VirtualKeyCode::F8 => Self::F8,
            VirtualKeyCode::F9 => Self::F9,
            VirtualKeyCode::F10 => Self::F10,
            VirtualKeyCode::F11 => Self::F11,
            VirtualKeyCode::F12 => Self::F12,
            VirtualKeyCode::F13 => Self::F13,
            VirtualKeyCode::F14 => Self::F14,
            VirtualKeyCode::F15 => Self::F15,
            VirtualKeyCode::F16 => Self::F16,
            VirtualKeyCode::F17 => Self::F17,
            VirtualKeyCode::F18 => Self::F18,
            VirtualKeyCode::F19 => Self::F19,
            VirtualKeyCode::F20 => Self::F20,
            VirtualKeyCode::F21 => Self::F21,
            VirtualKeyCode::F22 => Self::F22,
            VirtualKeyCode::F23 => Self::F23,
            VirtualKeyCode::F24 => Self::F24,
            VirtualKeyCode::Snapshot => Self::Snapshot,
            VirtualKeyCode::Scroll => Self::Scroll,
            VirtualKeyCode::Pause => Self::Pause,
            VirtualKeyCode::Insert => Self::Insert,
            VirtualKeyCode::Home => Self::Home,
            VirtualKeyCode::Delete => Self::Delete,
            VirtualKeyCode::End => Self::End,
            VirtualKeyCode::PageDown => Self::PageDown,
            VirtualKeyCode::PageUp => Self::PageUp,
            VirtualKeyCode::Left => Self::Left,
            VirtualKeyCode::Up => Self::Up,
            VirtualKeyCode::Right => Self::Right,
            VirtualKeyCode::Down => Self::Down,
            VirtualKeyCode::Back => Self::Backspace,
            VirtualKeyCode::Return => Self::Return,
            VirtualKeyCode::Space => Self::Space,
            VirtualKeyCode::Compose => Self::Compose,
            VirtualKeyCode::Caret => Self::Caret,
            VirtualKeyCode::Numlock => Self::Numlock,
            VirtualKeyCode::Numpad0 => Self::Numpad0,
            VirtualKeyCode::Numpad1 => Self::Numpad1,
            VirtualKeyCode::Numpad2 => Self::Numpad2,
            VirtualKeyCode::Numpad3 => Self::Numpad3,
            VirtualKeyCode::Numpad4 => Self::Numpad4,
            VirtualKeyCode::Numpad5 => Self::Numpad5,
            VirtualKeyCode::Numpad6 => Self::Numpad6,
            VirtualKeyCode::Numpad7 => Self::Numpad7,
            VirtualKeyCode::Numpad8 => Self::Numpad8,
            VirtualKeyCode::Numpad9 => Self::Numpad9,
            VirtualKeyCode::NumpadAdd => Self::NumpadAdd,
            VirtualKeyCode::NumpadDivide => Self::NumpadDivide,
            VirtualKeyCode::NumpadDecimal => Self::NumpadDecimal,
            VirtualKeyCode::NumpadComma => Self::NumpadComma,
            VirtualKeyCode::NumpadEnter => Self::NumpadEnter,
            VirtualKeyCode::NumpadEquals => Self::NumpadEquals,
            VirtualKeyCode::NumpadMultiply => Self::NumpadMultiply,
            VirtualKeyCode::NumpadSubtract => Self::NumpadSubtract,
            VirtualKeyCode::AbntC1 => Self::AbntC1,
            VirtualKeyCode::AbntC2 => Self::AbntC2,
            VirtualKeyCode::Apostrophe => Self::Apostrophe,
            VirtualKeyCode::Apps => Self::Apps,
            VirtualKeyCode::Asterisk => Self::Asterisk,
            VirtualKeyCode::At => Self::At,
            VirtualKeyCode::Ax => Self::Ax,
            VirtualKeyCode::Backslash => Self::Backslash,
            VirtualKeyCode::Calculator => Self::Calculator,
            VirtualKeyCode::Capital => Self::Capital,
            VirtualKeyCode::Colon => Self::Colon,
            VirtualKeyCode::Comma => Self::Comma,
            VirtualKeyCode::Convert => Self::Convert,
            VirtualKeyCode::Equals => Self::Equals,
            VirtualKeyCode::Grave => Self::Grave,
            VirtualKeyCode::Kana => Self::Kana,
            VirtualKeyCode::Kanji => Self::Kanji,
            VirtualKeyCode::LAlt => Self::LAlt,
            VirtualKeyCode::LBracket => Self::LBracket,
            VirtualKeyCode::LControl => Self::LControl,
            VirtualKeyCode::LShift => Self::LShift,
            VirtualKeyCode::LWin => Self::LWin,
            VirtualKeyCode::Mail => Self::Mail,
            VirtualKeyCode::MediaSelect => Self::MediaSelect,
            VirtualKeyCode::MediaStop => Self::MediaStop,
            VirtualKeyCode::Minus => Self::Minus,
            VirtualKeyCode::Mute => Self::Mute,
            VirtualKeyCode::MyComputer => Self::MyComputer,
            VirtualKeyCode::NavigateForward => Self::NavigateForward,
            VirtualKeyCode::NavigateBackward => Self::NavigateBackward,
            VirtualKeyCode::NextTrack => Self::NextTrack,
            VirtualKeyCode::NoConvert => Self::NoConvert,
            VirtualKeyCode::OEM102 => Self::OEM102,
            VirtualKeyCode::Period => Self::Period,
            VirtualKeyCode::PlayPause => Self::PlayPause,
            VirtualKeyCode::Plus => Self::Plus,
            VirtualKeyCode::Power => Self::Power,
            VirtualKeyCode::PrevTrack => Self::PrevTrack,
            VirtualKeyCode::RAlt => Self::RAlt,
            VirtualKeyCode::RBracket => Self::RBracket,
            VirtualKeyCode::RControl => Self::RControl,
            VirtualKeyCode::RShift => Self::RShift,
            VirtualKeyCode::RWin => Self::RWin,
            VirtualKeyCode::Semicolon => Self::Semicolon,
            VirtualKeyCode::Slash => Self::Slash,
            VirtualKeyCode::Sleep => Self::Sleep,
            VirtualKeyCode::Stop => Self::Stop,
            VirtualKeyCode::Sysrq => Self::Sysrq,
            VirtualKeyCode::Tab => Self::Tab,
            VirtualKeyCode::Underline => Self::Underline,
            VirtualKeyCode::Unlabeled => Self::Unlabeled,
            VirtualKeyCode::VolumeDown => Self::VolumeDown,
            VirtualKeyCode::VolumeUp => Self::VolumeUp,
            VirtualKeyCode::Wake => Self::Wake,
            VirtualKeyCode::WebBack => Self::WebBack,
            VirtualKeyCode::WebFavorites => Self::WebFavorites,
            VirtualKeyCode::WebForward => Self::WebForward,
            VirtualKeyCode::WebHome => Self::WebHome,
            VirtualKeyCode::WebRefresh => Self::WebRefresh,
            VirtualKeyCode::WebSearch => Self::WebSearch,
            VirtualKeyCode::WebStop => Self::WebStop,
            VirtualKeyCode::Yen => Self::Yen,
            VirtualKeyCode::Copy => Self::Copy,
            VirtualKeyCode::Paste => Self::Paste,
            VirtualKeyCode::Cut => Self::Cut,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum MouseInputType {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum LinearInputType {
    KnownKeyboard(KeyCode),
    Mouse(MouseInputType),
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum VectorInputType {
    MouseMove,
}

#[derive(Serialize, Deserialize)]
struct InputMapInner<TLinear, TVector> {
    linear_map: HashMap<LinearInputType, TLinear>,
    vector_map: HashMap<VectorInputType, TVector>,
}

/// Maps between physical inputs providable by the user, and whatever action representation
/// your game uses.
pub struct InputMap<TLinear, TVector> {
    inner: InputMapInner<TLinear, TVector>,
}

impl<TLinear, TVector> InputMap<TLinear, TVector> {
    pub fn empty() -> Self {
        Self {
            inner: InputMapInner {
                linear_map: HashMap::new(),
                vector_map: HashMap::new(),
            },
        }
    }

    pub fn assign_linear(&mut self, input: LinearInputType, value: TLinear) {
        self.inner.linear_map.insert(input, value);
    }

    pub fn unassign_linear(&mut self, input: &LinearInputType) {
        self.inner.linear_map.remove(input);
    }

    pub fn get_linear(&self, input: &LinearInputType) -> Option<&TLinear> {
        self.inner.linear_map.get(input)
    }

    pub fn assign_vector(&mut self, input: VectorInputType, value: TVector) {
        self.inner.vector_map.insert(input, value);
    }

    pub fn unassign_vector(&mut self, input: &VectorInputType) {
        self.inner.vector_map.remove(input);
    }

    pub fn get_vector(&self, input: &VectorInputType) -> Option<&TVector> {
        self.inner.vector_map.get(input)
    }
}

impl<TLinear: Serialize, TVector: Serialize> InputMap<TLinear, TVector> {
    pub fn to_str(&self) -> String {
        serde_json::to_string(&self.inner).expect("input map serialization failed")
    }
}

impl<'a, TLinear: Deserialize<'a>, TVector: Deserialize<'a>> InputMap<TLinear, TVector> {
    pub fn from_str(s: &'a str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            inner: serde_json::from_str(s)?,
        })
    }
}

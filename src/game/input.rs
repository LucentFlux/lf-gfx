use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    /// (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote,
    /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    /// 104- and 106-key layouts.
    /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash,
    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,
    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,
    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Digit0,
    /// <kbd>1</kbd> on a US keyboard.
    Digit1,
    /// <kbd>2</kbd> on a US keyboard.
    Digit2,
    /// <kbd>3</kbd> on a US keyboard.
    Digit3,
    /// <kbd>4</kbd> on a US keyboard.
    Digit4,
    /// <kbd>5</kbd> on a US keyboard.
    Digit5,
    /// <kbd>6</kbd> on a US keyboard.
    Digit6,
    /// <kbd>7</kbd> on a US keyboard.
    Digit7,
    /// <kbd>8</kbd> on a US keyboard.
    Digit8,
    /// <kbd>9</kbd> on a US keyboard.
    Digit9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,
    /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    /// Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash,
    /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo,
    /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    /// Russian keyboard.
    IntlYen,
    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    KeyA,
    /// <kbd>b</kbd> on a US keyboard.
    KeyB,
    /// <kbd>c</kbd> on a US keyboard.
    KeyC,
    /// <kbd>d</kbd> on a US keyboard.
    KeyD,
    /// <kbd>e</kbd> on a US keyboard.
    KeyE,
    /// <kbd>f</kbd> on a US keyboard.
    KeyF,
    /// <kbd>g</kbd> on a US keyboard.
    KeyG,
    /// <kbd>h</kbd> on a US keyboard.
    KeyH,
    /// <kbd>i</kbd> on a US keyboard.
    KeyI,
    /// <kbd>j</kbd> on a US keyboard.
    KeyJ,
    /// <kbd>k</kbd> on a US keyboard.
    KeyK,
    /// <kbd>l</kbd> on a US keyboard.
    KeyL,
    /// <kbd>m</kbd> on a US keyboard.
    KeyM,
    /// <kbd>n</kbd> on a US keyboard.
    KeyN,
    /// <kbd>o</kbd> on a US keyboard.
    KeyO,
    /// <kbd>p</kbd> on a US keyboard.
    KeyP,
    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    KeyQ,
    /// <kbd>r</kbd> on a US keyboard.
    KeyR,
    /// <kbd>s</kbd> on a US keyboard.
    KeyS,
    /// <kbd>t</kbd> on a US keyboard.
    KeyT,
    /// <kbd>u</kbd> on a US keyboard.
    KeyU,
    /// <kbd>v</kbd> on a US keyboard.
    KeyV,
    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    KeyW,
    /// <kbd>x</kbd> on a US keyboard.
    KeyX,
    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    KeyY,
    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    KeyZ,
    /// <kbd>-</kbd> on a US keyboard.
    Minus,
    /// <kbd>.</kbd> on a US keyboard.
    Period,
    /// <kbd>'</kbd> on a US keyboard.
    Quote,
    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,
    /// <kbd>/</kbd> on a US keyboard.
    Slash,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,
    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,
    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,
    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,
    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,
    /// <kbd> </kbd> (space)
    Space,
    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,
    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,
    /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
    KanaMode,
    /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    ///
    /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1,
    /// Korean: Hanja <kbd>한</kbd> (hanja)
    ///
    /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2,
    /// Japanese (word-processing keyboard): Katakana
    Lang3,
    /// Japanese (word-processing keyboard): Hiragana
    Lang4,
    /// Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5,
    /// Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert,
    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,
    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,
    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,
    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,
    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,
    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,
    /// <kbd>↓</kbd>
    ArrowDown,
    /// <kbd>←</kbd>
    ArrowLeft,
    /// <kbd>→</kbd>
    ArrowRight,
    /// <kbd>↑</kbd>
    ArrowUp,
    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,
    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,
    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
    Numpad1,
    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,
    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,
    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,
    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,
    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,
    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,
    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,
    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,
    /// <kbd>+</kbd>
    NumpadAdd,
    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,
    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,
    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,
    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,
    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,
    /// <kbd>/</kbd>
    NumpadDivide,
    NumpadEnter,
    /// <kbd>=</kbd>
    NumpadEqual,
    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,
    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,
    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,
    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,
    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,
    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,
    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,
    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,
    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,
    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,
    /// <kbd>-</kbd>
    NumpadSubtract,
    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,
    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,
    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,
    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,
    /// <kbd>Scroll Lock</kbd>
    ScrollLock,
    /// <kbd>Pause Break</kbd>
    Pause,
    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,
    BrowserFavorites,
    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,
    /// The "home" button on Android.
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    /// keyboards.
    Eject,
    /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1,
    /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    // Legacy modifier key. Also called "Super" in certain places.
    Meta,
    // Legacy modifier key.
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana,
    /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F13,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F14,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F15,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F16,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F17,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F18,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F19,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F20,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F21,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F22,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F23,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F24,
    /// General-purpose function key.
    F25,
    /// General-purpose function key.
    F26,
    /// General-purpose function key.
    F27,
    /// General-purpose function key.
    F28,
    /// General-purpose function key.
    F29,
    /// General-purpose function key.
    F30,
    /// General-purpose function key.
    F31,
    /// General-purpose function key.
    F32,
    /// General-purpose function key.
    F33,
    /// General-purpose function key.
    F34,
    /// General-purpose function key.
    F35,
}

impl From<winit::keyboard::KeyCode> for KeyCode {
    fn from(value: winit::keyboard::KeyCode) -> Self {
        match value {
            winit::keyboard::KeyCode::Backquote => Self::Backquote,
            winit::keyboard::KeyCode::Backslash => Self::Backslash,
            winit::keyboard::KeyCode::BracketLeft => Self::BracketLeft,
            winit::keyboard::KeyCode::BracketRight => Self::BracketRight,
            winit::keyboard::KeyCode::Comma => Self::Comma,
            winit::keyboard::KeyCode::Digit0 => Self::Digit0,
            winit::keyboard::KeyCode::Digit1 => Self::Digit1,
            winit::keyboard::KeyCode::Digit2 => Self::Digit2,
            winit::keyboard::KeyCode::Digit3 => Self::Digit3,
            winit::keyboard::KeyCode::Digit4 => Self::Digit4,
            winit::keyboard::KeyCode::Digit5 => Self::Digit5,
            winit::keyboard::KeyCode::Digit6 => Self::Digit6,
            winit::keyboard::KeyCode::Digit7 => Self::Digit7,
            winit::keyboard::KeyCode::Digit8 => Self::Digit8,
            winit::keyboard::KeyCode::Digit9 => Self::Digit9,
            winit::keyboard::KeyCode::Equal => Self::Equal,
            winit::keyboard::KeyCode::IntlBackslash => Self::IntlBackslash,
            winit::keyboard::KeyCode::IntlRo => Self::IntlRo,
            winit::keyboard::KeyCode::IntlYen => Self::IntlYen,
            winit::keyboard::KeyCode::KeyA => Self::KeyA,
            winit::keyboard::KeyCode::KeyB => Self::KeyB,
            winit::keyboard::KeyCode::KeyC => Self::KeyC,
            winit::keyboard::KeyCode::KeyD => Self::KeyD,
            winit::keyboard::KeyCode::KeyE => Self::KeyE,
            winit::keyboard::KeyCode::KeyF => Self::KeyF,
            winit::keyboard::KeyCode::KeyG => Self::KeyG,
            winit::keyboard::KeyCode::KeyH => Self::KeyH,
            winit::keyboard::KeyCode::KeyI => Self::KeyI,
            winit::keyboard::KeyCode::KeyJ => Self::KeyJ,
            winit::keyboard::KeyCode::KeyK => Self::KeyK,
            winit::keyboard::KeyCode::KeyL => Self::KeyL,
            winit::keyboard::KeyCode::KeyM => Self::KeyM,
            winit::keyboard::KeyCode::KeyN => Self::KeyN,
            winit::keyboard::KeyCode::KeyO => Self::KeyO,
            winit::keyboard::KeyCode::KeyP => Self::KeyP,
            winit::keyboard::KeyCode::KeyQ => Self::KeyQ,
            winit::keyboard::KeyCode::KeyR => Self::KeyR,
            winit::keyboard::KeyCode::KeyS => Self::KeyS,
            winit::keyboard::KeyCode::KeyT => Self::KeyT,
            winit::keyboard::KeyCode::KeyU => Self::KeyU,
            winit::keyboard::KeyCode::KeyV => Self::KeyV,
            winit::keyboard::KeyCode::KeyW => Self::KeyW,
            winit::keyboard::KeyCode::KeyX => Self::KeyX,
            winit::keyboard::KeyCode::KeyY => Self::KeyY,
            winit::keyboard::KeyCode::KeyZ => Self::KeyZ,
            winit::keyboard::KeyCode::Minus => Self::Minus,
            winit::keyboard::KeyCode::Period => Self::Period,
            winit::keyboard::KeyCode::Quote => Self::Quote,
            winit::keyboard::KeyCode::Semicolon => Self::Semicolon,
            winit::keyboard::KeyCode::Slash => Self::Slash,
            winit::keyboard::KeyCode::AltLeft => Self::AltLeft,
            winit::keyboard::KeyCode::AltRight => Self::AltRight,
            winit::keyboard::KeyCode::Backspace => Self::Backspace,
            winit::keyboard::KeyCode::CapsLock => Self::CapsLock,
            winit::keyboard::KeyCode::ContextMenu => Self::ContextMenu,
            winit::keyboard::KeyCode::ControlLeft => Self::ControlLeft,
            winit::keyboard::KeyCode::ControlRight => Self::ControlRight,
            winit::keyboard::KeyCode::Enter => Self::Enter,
            winit::keyboard::KeyCode::SuperLeft => Self::SuperLeft,
            winit::keyboard::KeyCode::SuperRight => Self::SuperRight,
            winit::keyboard::KeyCode::ShiftLeft => Self::ShiftLeft,
            winit::keyboard::KeyCode::ShiftRight => Self::ShiftRight,
            winit::keyboard::KeyCode::Space => Self::Space,
            winit::keyboard::KeyCode::Tab => Self::Tab,
            winit::keyboard::KeyCode::Convert => Self::Convert,
            winit::keyboard::KeyCode::KanaMode => Self::KanaMode,
            winit::keyboard::KeyCode::Lang1 => Self::Lang1,
            winit::keyboard::KeyCode::Lang2 => Self::Lang2,
            winit::keyboard::KeyCode::Lang3 => Self::Lang3,
            winit::keyboard::KeyCode::Lang4 => Self::Lang4,
            winit::keyboard::KeyCode::Lang5 => Self::Lang5,
            winit::keyboard::KeyCode::NonConvert => Self::NonConvert,
            winit::keyboard::KeyCode::Delete => Self::Delete,
            winit::keyboard::KeyCode::End => Self::End,
            winit::keyboard::KeyCode::Help => Self::Help,
            winit::keyboard::KeyCode::Home => Self::Home,
            winit::keyboard::KeyCode::Insert => Self::Insert,
            winit::keyboard::KeyCode::PageDown => Self::PageDown,
            winit::keyboard::KeyCode::PageUp => Self::PageUp,
            winit::keyboard::KeyCode::ArrowDown => Self::ArrowDown,
            winit::keyboard::KeyCode::ArrowLeft => Self::ArrowLeft,
            winit::keyboard::KeyCode::ArrowRight => Self::ArrowRight,
            winit::keyboard::KeyCode::ArrowUp => Self::ArrowUp,
            winit::keyboard::KeyCode::NumLock => Self::NumLock,
            winit::keyboard::KeyCode::Numpad0 => Self::Numpad0,
            winit::keyboard::KeyCode::Numpad1 => Self::Numpad1,
            winit::keyboard::KeyCode::Numpad2 => Self::Numpad2,
            winit::keyboard::KeyCode::Numpad3 => Self::Numpad3,
            winit::keyboard::KeyCode::Numpad4 => Self::Numpad4,
            winit::keyboard::KeyCode::Numpad5 => Self::Numpad5,
            winit::keyboard::KeyCode::Numpad6 => Self::Numpad6,
            winit::keyboard::KeyCode::Numpad7 => Self::Numpad7,
            winit::keyboard::KeyCode::Numpad8 => Self::Numpad8,
            winit::keyboard::KeyCode::Numpad9 => Self::Numpad9,
            winit::keyboard::KeyCode::NumpadAdd => Self::NumpadAdd,
            winit::keyboard::KeyCode::NumpadBackspace => Self::NumpadBackspace,
            winit::keyboard::KeyCode::NumpadClear => Self::NumpadClear,
            winit::keyboard::KeyCode::NumpadClearEntry => Self::NumpadClearEntry,
            winit::keyboard::KeyCode::NumpadComma => Self::NumpadComma,
            winit::keyboard::KeyCode::NumpadDecimal => Self::NumpadDecimal,
            winit::keyboard::KeyCode::NumpadDivide => Self::NumpadDivide,
            winit::keyboard::KeyCode::NumpadEnter => Self::NumpadEnter,
            winit::keyboard::KeyCode::NumpadEqual => Self::NumpadEqual,
            winit::keyboard::KeyCode::NumpadHash => Self::NumpadHash,
            winit::keyboard::KeyCode::NumpadMemoryAdd => Self::NumpadMemoryAdd,
            winit::keyboard::KeyCode::NumpadMemoryClear => Self::NumpadMemoryClear,
            winit::keyboard::KeyCode::NumpadMemoryRecall => Self::NumpadMemoryRecall,
            winit::keyboard::KeyCode::NumpadMemoryStore => Self::NumpadMemoryStore,
            winit::keyboard::KeyCode::NumpadMemorySubtract => Self::NumpadMemorySubtract,
            winit::keyboard::KeyCode::NumpadMultiply => Self::NumpadMultiply,
            winit::keyboard::KeyCode::NumpadParenLeft => Self::NumpadParenLeft,
            winit::keyboard::KeyCode::NumpadParenRight => Self::NumpadParenRight,
            winit::keyboard::KeyCode::NumpadStar => Self::NumpadStar,
            winit::keyboard::KeyCode::NumpadSubtract => Self::NumpadSubtract,
            winit::keyboard::KeyCode::Escape => Self::Escape,
            winit::keyboard::KeyCode::Fn => Self::Fn,
            winit::keyboard::KeyCode::FnLock => Self::FnLock,
            winit::keyboard::KeyCode::PrintScreen => Self::PrintScreen,
            winit::keyboard::KeyCode::ScrollLock => Self::ScrollLock,
            winit::keyboard::KeyCode::Pause => Self::Pause,
            winit::keyboard::KeyCode::BrowserBack => Self::BrowserBack,
            winit::keyboard::KeyCode::BrowserFavorites => Self::BrowserFavorites,
            winit::keyboard::KeyCode::BrowserForward => Self::BrowserForward,
            winit::keyboard::KeyCode::BrowserHome => Self::BrowserHome,
            winit::keyboard::KeyCode::BrowserRefresh => Self::BrowserRefresh,
            winit::keyboard::KeyCode::BrowserSearch => Self::BrowserSearch,
            winit::keyboard::KeyCode::BrowserStop => Self::BrowserStop,
            winit::keyboard::KeyCode::Eject => Self::Eject,
            winit::keyboard::KeyCode::LaunchApp1 => Self::LaunchApp1,
            winit::keyboard::KeyCode::LaunchApp2 => Self::LaunchApp2,
            winit::keyboard::KeyCode::LaunchMail => Self::LaunchMail,
            winit::keyboard::KeyCode::MediaPlayPause => Self::MediaPlayPause,
            winit::keyboard::KeyCode::MediaSelect => Self::MediaSelect,
            winit::keyboard::KeyCode::MediaStop => Self::MediaStop,
            winit::keyboard::KeyCode::MediaTrackNext => Self::MediaTrackNext,
            winit::keyboard::KeyCode::MediaTrackPrevious => Self::MediaTrackPrevious,
            winit::keyboard::KeyCode::Power => Self::Power,
            winit::keyboard::KeyCode::Sleep => Self::Sleep,
            winit::keyboard::KeyCode::AudioVolumeDown => Self::AudioVolumeDown,
            winit::keyboard::KeyCode::AudioVolumeMute => Self::AudioVolumeMute,
            winit::keyboard::KeyCode::AudioVolumeUp => Self::AudioVolumeUp,
            winit::keyboard::KeyCode::WakeUp => Self::WakeUp,
            winit::keyboard::KeyCode::Meta => Self::Meta,
            winit::keyboard::KeyCode::Hyper => Self::Hyper,
            winit::keyboard::KeyCode::Turbo => Self::Turbo,
            winit::keyboard::KeyCode::Abort => Self::Abort,
            winit::keyboard::KeyCode::Resume => Self::Resume,
            winit::keyboard::KeyCode::Suspend => Self::Suspend,
            winit::keyboard::KeyCode::Again => Self::Again,
            winit::keyboard::KeyCode::Copy => Self::Copy,
            winit::keyboard::KeyCode::Cut => Self::Cut,
            winit::keyboard::KeyCode::Find => Self::Find,
            winit::keyboard::KeyCode::Open => Self::Open,
            winit::keyboard::KeyCode::Paste => Self::Paste,
            winit::keyboard::KeyCode::Props => Self::Props,
            winit::keyboard::KeyCode::Select => Self::Select,
            winit::keyboard::KeyCode::Undo => Self::Undo,
            winit::keyboard::KeyCode::Hiragana => Self::Hiragana,
            winit::keyboard::KeyCode::Katakana => Self::Katakana,
            winit::keyboard::KeyCode::F1 => Self::F1,
            winit::keyboard::KeyCode::F2 => Self::F2,
            winit::keyboard::KeyCode::F3 => Self::F3,
            winit::keyboard::KeyCode::F4 => Self::F4,
            winit::keyboard::KeyCode::F5 => Self::F5,
            winit::keyboard::KeyCode::F6 => Self::F6,
            winit::keyboard::KeyCode::F7 => Self::F7,
            winit::keyboard::KeyCode::F8 => Self::F8,
            winit::keyboard::KeyCode::F9 => Self::F9,
            winit::keyboard::KeyCode::F10 => Self::F10,
            winit::keyboard::KeyCode::F11 => Self::F11,
            winit::keyboard::KeyCode::F12 => Self::F12,
            winit::keyboard::KeyCode::F13 => Self::F13,
            winit::keyboard::KeyCode::F14 => Self::F14,
            winit::keyboard::KeyCode::F15 => Self::F15,
            winit::keyboard::KeyCode::F16 => Self::F16,
            winit::keyboard::KeyCode::F17 => Self::F17,
            winit::keyboard::KeyCode::F18 => Self::F18,
            winit::keyboard::KeyCode::F19 => Self::F19,
            winit::keyboard::KeyCode::F20 => Self::F20,
            winit::keyboard::KeyCode::F21 => Self::F21,
            winit::keyboard::KeyCode::F22 => Self::F22,
            winit::keyboard::KeyCode::F23 => Self::F23,
            winit::keyboard::KeyCode::F24 => Self::F24,
            winit::keyboard::KeyCode::F25 => Self::F25,
            winit::keyboard::KeyCode::F26 => Self::F26,
            winit::keyboard::KeyCode::F27 => Self::F27,
            winit::keyboard::KeyCode::F28 => Self::F28,
            winit::keyboard::KeyCode::F29 => Self::F29,
            winit::keyboard::KeyCode::F30 => Self::F30,
            winit::keyboard::KeyCode::F31 => Self::F31,
            winit::keyboard::KeyCode::F32 => Self::F32,
            winit::keyboard::KeyCode::F33 => Self::F33,
            winit::keyboard::KeyCode::F34 => Self::F34,
            winit::keyboard::KeyCode::F35 => Self::F35,
            _ => todo!(),
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

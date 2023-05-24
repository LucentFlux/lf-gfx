use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Used to skip serialization on false fields
fn is_false(v: &bool) -> bool {
    !*v
}

/// Represents the current state of the keyboard modifiers
///
/// Each flag represents a modifier and is set if this modifier is active.
#[derive(Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct ModifiersState {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub shift: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub ctrl: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub alt: bool,
    /// This is the "windows" key on PC and "command" key on Mac.
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub logo: bool,
}

// Used to skip serialization on all-false modifiers
fn are_modifiers_all_false(v: &ModifiersState) -> bool {
    *v == ModifiersState::default()
}

/// A value between 0 and 1 that some input has been activated
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct InputActivation(f32);

impl InputActivation {
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
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct InputCode {
    /// Things that are held when the input occurred
    #[serde(default)]
    #[serde(skip_serializing_if = "are_modifiers_all_false")]
    pub modifiers: ModifiersState,
    /// The actual input that occurred to trigger this event
    #[serde(flatten)]
    pub inputted: InputType,
}

impl InputCode {
    pub fn unmodified(inputted: InputType) -> Self {
        Self {
            modifiers: ModifiersState::default(),
            inputted,
        }
    }
    pub fn key(key_code: winit::event::VirtualKeyCode) -> Self {
        Self {
            modifiers: ModifiersState::default(),
            inputted: InputType::KnownKeyboard(key_code),
        }
    }
}

// TODO: Integrate with a better input system, like steamworks
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum InputType {
    KnownKeyboard(winit::event::VirtualKeyCode),
    MouseMotion {
        delta: (u32, u32),
    },
    MouseWheel {
        delta: winit::dpi::PhysicalPosition<u32>,
    },
}

#[derive(Serialize, Deserialize)]
struct InputMapInner<T> {
    map: HashMap<InputCode, T>,
}

/// Maps between physical inputs providable by the user, and whatever action representation
/// your game uses.
pub struct InputMap<T> {
    inner: InputMapInner<T>,
}

impl<T> InputMap<T> {
    pub fn empty() -> Self {
        Self {
            inner: InputMapInner {
                map: HashMap::new(),
            },
        }
    }

    pub fn assign(&mut self, input: InputCode, value: T) {
        self.inner.map.insert(input, value);
    }

    pub fn unassign(&mut self, input: &InputCode) {
        self.inner.map.remove(input);
    }

    pub fn get(&self, input: &InputCode) -> Option<&T> {
        self.inner.map.get(input)
    }
}

impl<T: Serialize> InputMap<T> {
    pub fn to_str(&self) -> String {
        serde_json::to_string(&self.inner).expect("input map serialization failed")
    }
}

impl<'a, T: Deserialize<'a>> InputMap<T> {
    pub fn from_str(s: &'a str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            inner: serde_json::from_str(s)?,
        })
    }
}

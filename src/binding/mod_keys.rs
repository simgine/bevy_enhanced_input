use core::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bitflags::bitflags;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Keyboard modifiers for both left and right keys.
///
/// The number of keyboard modifiers in a [`Binding`](super::Binding) affects the
/// order in which its action is evaluated. See
/// [`ActionSettings::consume_input`](crate::prelude::ActionSettings::consume_input)
/// for more details.
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Debug, Default, PartialEq)
)]
#[cfg_attr(
    all(feature = "reflect", feature = "serialize"),
    reflect(Serialize, Deserialize)
)]
pub struct ModKeys(u8);

bitflags! {
    impl ModKeys: u8 {
        /// Corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
        const CONTROL = 0b00010001;
        /// Corresponds to [`KeyCode::ControlLeft`].
        const CONTROL_LEFT = 0b00000001;
        /// Corresponds to [`KeyCode::ControlRight`].
        const CONTROL_RIGHT = 0b00010000;
        /// Corresponds to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`].
        const SHIFT = 0b00100010;
        /// Corresponds to [`KeyCode::ShiftLeft`].
        const SHIFT_LEFT = 0b00000010;
        /// Corresponds to [`KeyCode::ShiftRight`].
        const SHIFT_RIGHT = 0b00100000;
        /// Corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
        const ALT = 0b01000100;
        /// Corresponds to [`KeyCode::AltLeft`].
        const ALT_LEFT = 0b00000100;
        /// Corresponds to [`KeyCode::AltRight`].
        const ALT_RIGHT = 0b01000000;
        /// Corresponds to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
        const SUPER = 0b10001000;
        /// Corresponds to [`KeyCode::SuperLeft`].
        const SUPER_LEFT = 0b00001000;
        /// Corresponds to [`KeyCode::SuperRight`].
        const SUPER_RIGHT = 0b10000000;
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ModKeys {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        bitflags::serde::serialize(self, serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de> Deserialize<'de> for ModKeys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bitflags::serde::deserialize(deserializer)
    }
}

impl Display for ModKeys {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (index, (_, mod_key)) in self.iter_names().enumerate() {
            if index != 0 {
                write!(f, " + ")?;
            }
            match mod_key {
                ModKeys::CONTROL => write!(f, "Ctrl")?,
                ModKeys::CONTROL_LEFT => write!(f, "LeftCtrl")?,
                ModKeys::CONTROL_RIGHT => write!(f, "RightCtrl")?,
                ModKeys::SHIFT => write!(f, "Shift")?,
                ModKeys::SHIFT_LEFT => write!(f, "LeftShift")?,
                ModKeys::SHIFT_RIGHT => write!(f, "RightShift")?,
                ModKeys::ALT => write!(f, "Alt")?,
                ModKeys::ALT_LEFT => write!(f, "LeftAlt")?,
                ModKeys::ALT_RIGHT => write!(f, "RightAlt")?,
                ModKeys::SUPER => write!(f, "Super")?,
                ModKeys::SUPER_LEFT => write!(f, "LeftSuper")?,
                ModKeys::SUPER_RIGHT => write!(f, "RightSuper")?,
                _ => unreachable!("iteration should yield only named flags"),
            }
        }

        Ok(())
    }
}

impl ModKeys {
    /// Returns an instance with currently active modifiers.
    #[must_use]
    pub fn pressed(keys: &ButtonInput<KeyCode>, unique_side: bool) -> Self {
        let mut mod_keys = Self::empty();
        for modifier in Self::all().iter_keys() {
            modifier.into_iter().for_each(|key: KeyCode| {
                if keys.pressed(key) {
                    let mod_key: ModKeys = key.into();
                    mod_keys |= mod_key;
                    if !unique_side {
                        if ModKeys::CONTROL.intersects(mod_key) {
                            mod_keys |= ModKeys::CONTROL;
                        } else if ModKeys::SHIFT.intersects(mod_key) {
                            mod_keys |= ModKeys::SHIFT;
                        } else if ModKeys::ALT.intersects(mod_key) {
                            mod_keys |= ModKeys::ALT;
                        } else if ModKeys::SUPER.intersects(mod_key) {
                            mod_keys |= ModKeys::SUPER;
                        }
                    } else {
                        mod_keys |= mod_key;
                    }
                }
            });
        }

        mod_keys
    }

    /// Returns an iterator over the key codes corresponding to the set modifier bits.
    pub fn iter_keys(self) -> impl Iterator<Item = Vec<KeyCode>> {
        self.iter_names().map(|(_, mod_key)| match mod_key {
            ModKeys::CONTROL => vec![KeyCode::ControlLeft, KeyCode::ControlRight],
            ModKeys::CONTROL_LEFT => vec![KeyCode::ControlLeft],
            ModKeys::CONTROL_RIGHT => vec![KeyCode::ControlRight],
            ModKeys::SHIFT => vec![KeyCode::ShiftLeft, KeyCode::ShiftRight],
            ModKeys::SHIFT_LEFT => vec![KeyCode::ShiftLeft],
            ModKeys::SHIFT_RIGHT => vec![KeyCode::ShiftRight],
            ModKeys::ALT => vec![KeyCode::AltLeft, KeyCode::AltRight],
            ModKeys::ALT_LEFT => vec![KeyCode::AltLeft],
            ModKeys::ALT_RIGHT => vec![KeyCode::AltRight],
            ModKeys::SUPER => vec![KeyCode::SuperLeft, KeyCode::SuperRight],
            ModKeys::SUPER_LEFT => vec![KeyCode::SuperLeft],
            ModKeys::SUPER_RIGHT => vec![KeyCode::SuperRight],
            _ => unreachable!("iteration should yield only named flags"),
        })
    }
}

impl From<KeyCode> for ModKeys {
    /// Converts key into a named modifier
    ///
    /// Returns [`ModKeys::empty`] if the key is not a modifier.
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::ControlLeft => ModKeys::CONTROL_LEFT,
            KeyCode::ControlRight => ModKeys::CONTROL_RIGHT,
            KeyCode::ShiftLeft => ModKeys::SHIFT_LEFT,
            KeyCode::ShiftRight => ModKeys::SHIFT_RIGHT,
            KeyCode::AltLeft => ModKeys::ALT_LEFT,
            KeyCode::AltRight => ModKeys::ALT_RIGHT,
            KeyCode::SuperLeft => ModKeys::SUPER_LEFT,
            KeyCode::SuperRight => ModKeys::SUPER_RIGHT,
            _ => ModKeys::empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn pressed_mod_keys() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::KeyC);

        // Checking for either key
        let mod_keys = ModKeys::pressed(&keys, false);
        assert!(mod_keys.intersects(ModKeys::CONTROL));
        assert!(mod_keys.intersects(ModKeys::SHIFT));
        assert_eq!(mod_keys, ModKeys::CONTROL | ModKeys::SHIFT);
        // This is currently not equal to either key
        assert_ne!(mod_keys, ModKeys::CONTROL_LEFT | ModKeys::SHIFT_LEFT);

        // Checking for unique key
        let mod_keys = ModKeys::pressed(&keys, true);
        assert_eq!(mod_keys, ModKeys::CONTROL_LEFT | ModKeys::SHIFT_LEFT);
        assert_ne!(mod_keys, ModKeys::CONTROL_RIGHT | ModKeys::SHIFT_RIGHT);
        // This is currently not equal to the unique keys
        assert_ne!(mod_keys, ModKeys::CONTROL | ModKeys::SHIFT);
        // Even though it intersects with them
        assert!(mod_keys.intersects(ModKeys::CONTROL));
        assert!(mod_keys.intersects(ModKeys::SHIFT));
    }

    #[test]
    fn mod_keys_display() {
        assert_eq!(ModKeys::CONTROL.to_string(), "Ctrl");
        assert_eq!(ModKeys::all().to_string(), "Ctrl + Shift + Alt + Super");
        assert_eq!(ModKeys::empty().to_string(), "");
        assert_eq!(ModKeys::ALT_LEFT.to_string(), "LeftAlt");
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn mod_keys_serde() {
        assert_eq!(ron::to_string(&ModKeys::CONTROL).unwrap(), "\"CONTROL\"");
        assert_eq!(
            ron::to_string(&(ModKeys::CONTROL | ModKeys::SHIFT)).unwrap(),
            "\"CONTROL | SHIFT\""
        );
        assert_eq!(ron::to_string(&ModKeys::empty()).unwrap(), "\"\"");
        assert_eq!(
            ron::to_string(&ModKeys::SUPER_LEFT).unwrap(),
            "\"LEFT_SUPER\""
        );

        let parsed: ModKeys = ron::from_str("\"CONTROL | SHIFT\"").unwrap();
        assert_eq!(parsed, ModKeys::CONTROL | ModKeys::SHIFT);

        let parsed: ModKeys = ron::from_str("\"RIGHT_ALT | RIGHT_SUPER\"").unwrap();
        assert_eq!(parsed, ModKeys::ALT_RIGHT | ModKeys::SUPER_RIGHT);
    }
}

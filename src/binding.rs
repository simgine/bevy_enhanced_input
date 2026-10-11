//! Input bindings define which physical inputs map to an action.
//!
//! These are bound to actions using the [`BindingOf`] relationship,
//! which can be created using the [`bindings!`] macro, spawned underneath an [action entity](crate::action).
//!
//! Inputs can be modified using [input modifiers](crate::modifier) to alter the input value captured,
//! or [input conditions](crate::condition) to change when the action is triggered.
//!
//! When defining input bindings, you may find the collection of [preset bindings](crate::preset) useful
//! to reduce boilerplate and demonstrate common input patterns and transformations.
//!
//! For an exhaustive list of available input devices, see the [`Binding`] enum.

pub mod mod_keys;
pub mod relationship;

use std::fmt::{self, Display, Formatter};

use bevy::{
    ecs::{
        lifecycle::HookContext,
        template::{Template, TemplateContext},
        world::DeferredWorld,
    },
    log::{Level, error, tracing::enabled, warn},
    prelude::*,
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// A an input bound to an [`Action<C>`].
///
/// Should be stored on a separate entity from the action,
/// and related to it using the [`BindingOf`] relationship.
///
/// Buttons pressed and released within a single frame are still captured as active
/// when the context is evaluated during that frame. Their release is observed on
/// the next evaluation after Bevy clears its per-frame input flags, provided they
/// remain inactive.
///
/// [Input modifiers](crate::modifier) can change the captured dimension.
///
/// If the action's dimension differs from the captured input, it will be converted using
/// [`ActionValue::convert`](crate::action::value::ActionValue::convert).
#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, PartialEq)
)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(
    all(feature = "reflect", feature = "serialize"),
    reflect(Serialize, Deserialize)
)]
#[component(on_insert = on_insert, immutable)]
#[require(FirstActivation)]
pub enum Binding {
    /// Keyboard button, captured as [`ActionValue::Bool`].
    Keyboard { key: KeyCode, mod_keys: ModKeys },
    /// Mouse button, captured as [`ActionValue::Bool`].
    MouseButton {
        button: MouseButton,
        mod_keys: ModKeys,
    },
    /// Mouse movement, captured as [`ActionValue::Axis2D`].
    MouseMotion { mod_keys: ModKeys },
    /// Mouse wheel, captured as [`ActionValue::Axis2D`].
    ///
    /// <div class="warning">
    ///
    /// In Bevy, vertical scrolling maps to the Y axis. If your action output
    /// is [`f32`], you will get only horizontal scrolling by default.
    ///
    /// </div>
    ///
    /// # Examples
    ///
    /// Apply [`SwizzleAxis::YXZ`] modifier to get vertical scrolling for an
    /// action with an [`f32`] output.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_enhanced_input::prelude::*;
    ///
    /// let scene = bsn! {
    ///     PlayerCam
    ///     Actions<PlayerCam> [
    ///         Action::<Zoom>::new()
    ///         Bindings [
    ///             Binding::mouse_wheel()
    ///             // Without this modifier, it will use horizontal mouse scrolling.
    ///             ~SwizzleAxis::YXZ
    ///         ]
    ///     ]
    /// };
    ///
    /// #[derive(InputAction)]
    /// #[action_output(f32)]
    /// struct Zoom;
    ///
    /// #[derive(Component, Default, Clone)]
    /// struct PlayerCam;
    /// ```
    MouseWheel { mod_keys: ModKeys },
    /// Gamepad button, captured as [`ActionValue::Axis1D`].
    GamepadButton(GamepadButton),
    /// Gamepad stick axis, captured as [`ActionValue::Axis1D`].
    GamepadAxis(GamepadAxis),
    /// Any key, mouse button, or gamepad button, captured as [`ActionValue::Bool`].
    ///
    /// If used with a context with [`GamepadDevice::Single`], it will only
    /// activate on inputs from that gamepad in addition to mouse and keyboard.
    ///
    /// If [`ActionSettings::consume_input`] is set, this binding consumes all button
    /// inputs, not just the one that activated it. To have an action with this binding
    /// evaluated first, place it in a higher-priority context.
    AnyKey,
    /// Identifier of the input entry in [`CustomInputs`] to read.
    ///
    /// Use this for inputs that can't be represented or feed into Bevy input resources.
    /// Obtain an identifier with [`CustomInputs::register_input`].
    Custom(CustomInput),
    /// Doesn't correspond to any input, captured as [`ActionValue::Bool`] with `false`.
    ///
    /// Useful for expressing empty bindings in [presets](crate::preset).
    None,
}

impl Binding {
    /// Returns [`Self::MouseMotion`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self::MouseMotion {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns [`Self::MouseWheel`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self::MouseWheel {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns the amount of associated keyboard modifiers.
    #[must_use]
    pub fn mod_keys_count(self) -> usize {
        self.mod_keys().iter_names().count()
    }

    /// Returns associated keyboard modifiers.
    #[must_use]
    pub const fn mod_keys(self) -> ModKeys {
        match self {
            Binding::Keyboard { mod_keys, .. }
            | Binding::MouseButton { mod_keys, .. }
            | Binding::MouseMotion { mod_keys }
            | Binding::MouseWheel { mod_keys } => mod_keys,
            Binding::GamepadButton(_)
            | Binding::GamepadAxis(_)
            | Binding::AnyKey
            | Binding::Custom(_)
            | Binding::None => ModKeys::empty(),
        }
    }

    /// Returns new instance without any keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Self::GamepadButton`] or [`Self::GamepadAxis`].
    #[must_use]
    pub fn without_mod_keys(self) -> Self {
        self.with_mod_keys(ModKeys::empty())
    }
}

impl FromTemplate for Binding {
    type Template = BindingTemplate;
}

/// BSN template for a [`Binding`].
///
/// Its tuple field accepts values convertible into [`Binding`], enabling declarations like
/// `Binding(KeyCode::Space)` and `Binding(GamepadButton::South)` inside BSN.
/// Use `Binding(Binding::None)` or `Binding(Binding::AnyKey)` for explicit enum values.
/// An omitted value defaults to [`Binding::None`].
#[derive(Debug, PartialEq)]
pub struct BindingTemplate(pub Binding);

impl Default for BindingTemplate {
    fn default() -> Self {
        Self(Binding::None)
    }
}

impl<I: Into<Binding>> From<I> for BindingTemplate {
    fn from(input: I) -> Self {
        Self(input.into())
    }
}

impl BindingTemplate {
    /// Returns a mouse motion template without keyboard modifiers.
    ///
    /// See also [`Binding::mouse_motion`].
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self(Binding::mouse_motion())
    }

    /// Returns a mouse wheel template without keyboard modifiers.
    ///
    /// See also [`Binding::mouse_wheel`].
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self(Binding::mouse_wheel())
    }

    /// Returns a template with assigned keyboard modifiers.
    ///
    /// See also [`InputModKeys::with_mod_keys`].
    #[must_use]
    pub fn with_mod_keys(self, mod_keys: ModKeys) -> Self {
        Self(self.0.with_mod_keys(mod_keys))
    }

    /// Returns a template without keyboard modifiers.
    ///
    /// See also [`Binding::without_mod_keys`].
    #[must_use]
    pub fn without_mod_keys(self) -> Self {
        Self(self.0.without_mod_keys())
    }
}

impl Template for BindingTemplate {
    type Output = Binding;

    fn build_template(&self, _context: &mut TemplateContext) -> Result<Binding> {
        Ok(self.0)
    }

    fn clone_template(&self) -> Self {
        Self(self.0)
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mod_keys = self.mod_keys();
        if !mod_keys.is_empty() {
            write!(f, "{mod_keys} + ")?;
        }

        match self {
            Binding::Keyboard { key, .. } => write!(f, "{key:?}"),
            Binding::MouseButton { button, .. } => write!(f, "Mouse {button:?}"),
            Binding::MouseMotion { .. } => write!(f, "Mouse Motion"),
            Binding::MouseWheel { .. } => write!(f, "Scroll Wheel"),
            Binding::GamepadButton(gamepad_button) => write!(f, "{gamepad_button:?}"),
            Binding::GamepadAxis(gamepad_axis) => write!(f, "{gamepad_axis:?}"),
            Binding::AnyKey => write!(f, "Any Key"),
            Binding::Custom(input) => write!(f, "{input:?}"),
            Binding::None => write!(f, "None"),
        }
    }
}

impl From<KeyCode> for Binding {
    fn from(key: KeyCode) -> Self {
        Self::Keyboard {
            key,
            mod_keys: Default::default(),
        }
    }
}

impl From<MouseButton> for Binding {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            mod_keys: Default::default(),
        }
    }
}

impl From<GamepadButton> for Binding {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl From<GamepadAxis> for Binding {
    fn from(value: GamepadAxis) -> Self {
        Self::GamepadAxis(value)
    }
}

/// A trait to ergonomically assign keyboard modifiers to any type that can be converted into a [`Binding`].
pub trait InputModKeys {
    /// Returns a binding with assigned keyboard modifiers.
    #[must_use]
    fn with_mod_keys(self, mod_keys: ModKeys) -> Binding;
}

impl<I: Into<Binding>> InputModKeys for I {
    /// Returns new instance with the replaced keyboard modifiers.
    ///
    /// Prints error and does nothing when called on [`Binding::GamepadButton`],
    /// [`Binding::GamepadAxis`], [`Binding::AnyKey`] or [`Binding::None`].
    fn with_mod_keys(self, mod_keys: ModKeys) -> Binding {
        let binding = self.into();
        match binding {
            Binding::Keyboard { key, .. } => Binding::Keyboard { key, mod_keys },
            Binding::MouseButton { button, .. } => Binding::MouseButton { button, mod_keys },
            Binding::MouseMotion { .. } => Binding::MouseMotion { mod_keys },
            Binding::MouseWheel { .. } => Binding::MouseWheel { mod_keys },
            Binding::GamepadButton { .. }
            | Binding::GamepadAxis { .. }
            | Binding::None
            | Binding::AnyKey
            | Binding::Custom(_) => {
                error!("can't add `{mod_keys:?}` to `{binding:?}`");
                binding
            }
        }
    }
}

fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
    let mut entity = world.entity_mut(ctx.entity);

    let mut first_activation = entity.get_mut::<FirstActivation>().unwrap();
    **first_activation = true;

    if enabled!(Level::WARN) {
        if let Some(action) = entity.get::<BindingOf>().map(|b| **b) {
            if world.get::<TriggerState>(action).is_none() {
                let binding = world.get::<Binding>(ctx.entity).unwrap();
                warn!(
                    "`{}` has binding `{binding:?}`, but the associated action `{action}` is invalid",
                    ctx.entity
                );
            }
        } else {
            let binding = world.get::<Binding>(ctx.entity).unwrap();
            warn!(
                "`{}` has binding `{binding:?}`, but it is not associated with any action",
                ctx.entity
            );
        }
    }
}

/// Tracks whether the input defined by [`Binding`] was active at least once.
///
/// Used to prevent newly created contexts from reacting to currently active inputs
/// until they're released.
///
/// Used only if [`ActionSettings::require_reset`] is set.
#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct FirstActivation(bool);

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use super::*;

    #[test]
    fn input_display() {
        assert_eq!(
            Binding::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "KeyA"
        );
        assert_eq!(
            Binding::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::CONTROL
            }
            .to_string(),
            "Ctrl + KeyA"
        );
        assert_eq!(
            Binding::MouseButton {
                button: MouseButton::Left,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Left"
        );
        assert_eq!(
            Binding::MouseMotion {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Motion"
        );
        assert_eq!(
            Binding::MouseWheel {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Scroll Wheel"
        );
        assert_eq!(
            Binding::GamepadAxis(GamepadAxis::LeftStickX).to_string(),
            "LeftStickX"
        );
        assert_eq!(
            Binding::GamepadButton(GamepadButton::North).to_string(),
            "North"
        );
    }
}

//! Values for [`Binding::Custom`] entries.

use bevy::{platform::collections::HashMap, prelude::*};

use crate::prelude::*;

/// Stores values for [`Binding::Custom`] entries.
///
/// Write to this resource from any system. A missing entry reads as
/// [`ActionValue::Bool`] `false`.
///
/// # Examples
///
/// ```
/// use bevy::{input::gestures::PinchGesture, prelude::*};
/// use bevy_enhanced_input::prelude::*;
///
/// fn stage_pinch(
///     mut events: MessageReader<PinchGesture>,
///     mut customs: ResMut<CustomInputs>,
/// ) {
///     let delta: f32 = events.read().map(|e| e.0).sum();
///     customs.insert("pinch", ActionValue::Axis1D(delta));
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct CustomInputs(HashMap<&'static str, ActionValue>);

impl CustomInputs {
    /// Sets the value for `name`.
    pub fn insert(&mut self, name: &'static str, value: ActionValue) {
        self.0.insert(name, value);
    }

    /// Returns the value for `name`, if any.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ActionValue> {
        self.0.get(name).copied()
    }

    /// Removes the value for `name`.
    pub fn remove(&mut self, name: &str) {
        self.0.remove(name);
    }

    /// Removes all values.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

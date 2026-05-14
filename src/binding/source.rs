//! Stores values for [`Binding::Custom`] entries.

use bevy::{platform::collections::HashMap, prelude::*};

use crate::prelude::*;

/// Write to this resource from any system to make custom input values available to actions.
///
/// Missing entries are read as [`ActionValue::Bool`] `false`.
///
/// # Examples
///
/// Feeding trackpad pinch events into a `Custom("pinch")` binding:
///
/// ```
/// use bevy::{input::gestures::PinchGesture, prelude::*};
/// use bevy_enhanced_input::prelude::*;
///
/// let mut app = App::new();
/// app.add_systems(
///     PreUpdate,
///     stage_pinch
///         .after(bevy::input::InputSystems)
///         .before(EnhancedInputSystems::Update),
/// );
///
/// fn stage_pinch(mut events: MessageReader<PinchGesture>, mut customs: ResMut<CustomInputs>) {
///     let delta: f32 = events.read().map(|e| e.0).sum();
///     customs.insert("pinch", ActionValue::Axis1D(delta));
/// }
/// ```
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct CustomInputs(HashMap<&'static str, ActionValue>);

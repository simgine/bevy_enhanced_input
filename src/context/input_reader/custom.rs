//! Identifiers and storage for [`Binding::Custom`] values.

use bevy::{platform::collections::HashMap, prelude::*};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Identifier for a custom input, used in [`Binding::Custom`].
///
/// Obtainable from [`CustomInputs::register_input`].
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Debug, Hash, PartialEq)
)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(
    all(feature = "reflect", feature = "serialize"),
    reflect(Serialize, Deserialize)
)]
pub struct CustomInput(usize);

/// Stores values for [`Binding::Custom`] entries.
///
/// Write to this resource from any system to make custom input values available to actions.
///
/// To register a custom input, use [`Self::register_input`].
///
/// Missing entries are read as [`ActionValue::Bool`] `false`.
///
/// # Examples
///
/// Feeding trackpad pinch events:
///
/// ```
/// use bevy::{input::gestures::PinchGesture, prelude::*};
/// use bevy_enhanced_input::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((MinimalPlugins, EnhancedInputPlugin));
/// let pinch = app
///     .world_mut()
///     .resource_mut::<CustomInputs>()
///     .register_input();
/// app.insert_resource(PinchId(pinch)).add_systems(
///     PreUpdate,
///     stage_pinch
///         .after(bevy::input::InputSystems)
///         .before(EnhancedInputSystems::Update),
/// );
///
/// fn stage_pinch(
///     mut events: MessageReader<PinchGesture>,
///     mut custom_inputs: ResMut<CustomInputs>,
///     id: Res<PinchId>,
/// ) {
///     let delta: f32 = events.read().map(|e| e.0).sum();
///     custom_inputs.insert(id.0, ActionValue::Axis1D(delta));
/// }
///
/// #[derive(Resource)]
/// struct PinchId(CustomInput);
/// ```
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct CustomInputs {
    #[deref]
    map: HashMap<CustomInput, ActionValue>,
    counter: usize,
}

impl CustomInputs {
    /// Creates a new custom input identifier.
    ///
    /// IDs are assigned in registration order, so two runs that register in
    /// the same order produce the same IDs.
    #[must_use]
    pub fn register_input(&mut self) -> CustomInput {
        let id = CustomInput(self.counter);
        self.counter += 1;
        id
    }
}

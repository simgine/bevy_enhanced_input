//! Identifiers and storage for [`Binding::Custom`] entries.

use core::sync::atomic::{AtomicUsize, Ordering};

use bevy::{platform::collections::HashMap, prelude::*};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Identifier for a custom input, used as the payload of [`Binding::Custom`].
///
/// Obtain one with [`CustomInput::register_input`] at startup and reuse it
/// when binding and when writing values into [`CustomInputs`].
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
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct CustomInput(usize);

impl CustomInput {
    /// Returns a fresh identifier.
    ///
    /// Each call increments a process-wide counter. IDs are assigned in
    /// registration order, so two runs that register in the same order
    /// produce the same IDs.
    #[must_use]
    pub fn register_input() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Write to this resource from any system to make custom input values available to actions.
///
/// Missing entries are read as [`ActionValue::Bool`] `false`.
///
/// # Examples
///
/// Feeding trackpad pinch events into a `Binding::Custom` entry:
///
/// ```
/// use bevy::{input::gestures::PinchGesture, prelude::*};
/// use bevy_enhanced_input::prelude::*;
///
/// #[derive(Resource)]
/// struct PinchId(CustomInput);
///
/// let mut app = App::new();
/// let pinch = CustomInput::register_input();
/// app.insert_resource(PinchId(pinch)).add_systems(
///     PreUpdate,
///     stage_pinch
///         .after(bevy::input::InputSystems)
///         .before(EnhancedInputSystems::Update),
/// );
///
/// fn stage_pinch(
///     mut events: MessageReader<PinchGesture>,
///     mut customs: ResMut<CustomInputs>,
///     id: Res<PinchId>,
/// ) {
///     let delta: f32 = events.read().map(|e| e.0).sum();
///     customs.insert(id.0, ActionValue::Axis1D(delta));
/// }
/// ```
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct CustomInputs(HashMap<CustomInput, ActionValue>);

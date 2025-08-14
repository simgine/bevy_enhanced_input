/*!
[`SpawnableList`](bevy::ecs::spawn::SpawnableList)s with common modifiers.

Similar to other [`SpawnableList`](bevy::ecs::spawn::SpawnableList)s in Bevy, like [`SpawnWith`](bevy::ecs::spawn::SpawnWith)
or [`SpawnIter`](bevy::ecs::spawn::SpawnIter), you need to call [`SpawnRelated::spawn`](bevy::prelude::SpawnRelated)
implemented for [`Bindings`](crate::prelude::Bindings) directly instead of using the [`bindings!`](crate::prelude::bindings) macro.

# Examples

With additional bindings.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
Bindings::spawn((
    Cardinal::wasd_keys(),
    Axial::left_stick(),
    // Additional bindings needs to use `Binding::from` wrapped
    // into `Spawn`, which is what `bindings!` macro does.
    Spawn((Binding::from(KeyCode::ArrowUp), SwizzleAxis::YXZ))
));
```

Initializing fields.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
Bindings::spawn((
    Bidirectional {
        // Struct fields are bundles, so you can also attach modifiers to individual fields.
        positive: (Binding::from(KeyCode::NumpadAdd), Scale::splat(2.0)),
        negative: Binding::from(KeyCode::NumpadSubtract),
    },
    Axial::left_stick().with((Scale::splat(1.0), SmoothNudge::default())), // Attach components to each field.
));
```

Loading from settings.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# use serde::{Serialize, Deserialize};
// Could be loaded from a file.
// `Binding::None` represents unbound inputs.
let settings = InputSettings {
    forward: [Binding::from(KeyCode::KeyW), Binding::None],
    right: [Binding::from(KeyCode::KeyA), Binding::None],
    backward: [Binding::from(KeyCode::KeyS), Binding::None],
    left: [Binding::from(KeyCode::KeyD), Binding::None],
};

Bindings::spawn((
    Cardinal {
        north: settings.forward[0],
        east: settings.right[0],
        south: settings.backward[0],
        west: settings.left[0],
    },
    Cardinal {
        north: settings.forward[1],
        east: settings.right[1],
        south: settings.backward[1],
        west: settings.left[1],
    },
));

/// Bindings for actions.
///
/// Represented as arrays because in games you usually
/// have 2 or 3 bindings for a single action.
///
/// Usually stored as a resource.
#[derive(Resource, Serialize, Deserialize)]
struct InputSettings {
    forward: [Binding; 2],
    right: [Binding; 2],
    backward: [Binding; 2],
    left: [Binding; 2],
}
```
*/

pub mod axial;
pub mod bidirectional;
pub mod cardinal;
pub mod ordinal;
pub mod spatial;

/// Helper trait for attaching a bundle to a preset.
///
/// See the module documentation for a usage example.
pub trait WithBundle<T> {
    type Output;

    /// Returns a new instance where the given bundle is added to each preset bundle.
    ///
    /// # Examples
    ///
    /// Be careful when attaching modifiers like [`SwizzleAxis`](crate::prelude::SwizzleAxis)
    /// or [`Negate`](crate::prelude::Negate), as they might already be used by the preset,
    /// which will result in a duplicate component panic.
    ///
    /// ```should_panic
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut world = World::new();
    /// world.spawn(Bindings::spawn(
    ///     Bidirectional::left_right_dpad().with(Negate::x()),
    /// ));
    /// ```
    ///
    /// To avoid this, you can attach such modifiers at the action level.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut world = World::new();
    /// world.spawn((
    ///     Action::<Move>::new(),
    ///     Negate::x(),
    ///     Bindings::spawn(Bidirectional::left_right_dpad()),
    /// ));
    ///
    /// #[derive(InputAction)]
    /// #[action_output(f32)]
    /// struct Move;
    /// ```
    fn with(self, bundle: T) -> Self::Output;
}

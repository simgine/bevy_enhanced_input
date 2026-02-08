/*!
Preset bindings for common input binding patterns.

Consider a common pattern of moving a character using keyboard keys and a gamepad stick.
With the help of [modifiers](crate::modifier) you can achieve this as follows:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, TypePath)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Movement>::new(),
            // Modifier components at the action level.
            DeadZone::default(),    // Applies non-uniform normalization.
            SmoothNudge::default(), // Smoothes movement.
            bindings![
                // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
                // so you need to assign keys to axes using swizzle to reorder them and negation.
                (KeyCode::KeyW, SwizzleAxis::YXZ),
                (KeyCode::KeyA, Negate::all()),
                (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
                KeyCode::KeyD,
                // In Bevy sticks split by axes and captured as 1-dimensional inputs,
                // so Y stick needs to be sweezled into Y axis.
                GamepadAxis::LeftStickX,
                (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
            ]
        ),
    ]),
));
```

However, this is quite onerous! It would be inconvenient to bind WASD keys and analog sticks manually, like in the example above,
every time. You can use [`Cardinal`](crate::Cardinal) and [`Axial`](crate::Axial) presets to simplify the example above.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
let mut world = World::new();

#[derive(Component, TypePath)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Movement>::new(),
            DeadZone::default(),
            SmoothNudge::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
            )),
        ),
    ]),
));
```

# Implementation details

Each of the preset types defined in this module generates a list of binding entities with the appropriate components
attached to them.

To achieve this, each preset type implements the [`SpawnableList`](bevy::ecs::spawn::SpawnableList) trait.
Similar to other [`SpawnableList`](bevy::ecs::spawn::SpawnableList)s in Bevy, like [`SpawnWith`](bevy::ecs::spawn::SpawnWith)
or [`SpawnIter`](bevy::ecs::spawn::SpawnIter), you need to call [`Bindings::spawn`](bevy::prelude::SpawnRelated)
from the [`SpawnRelated`](bevy::prelude::SpawnRelated) trait to generate the binding entities.

You cannot use the [`bindings!`](crate::prelude::bindings) macro.

# Examples

Adding additional bindings:

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

Initializing fields:

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

Loading from settings:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
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
#[derive(Resource)]
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
pub trait WithBundle<B> {
    type Output;

    /// Returns a new instance in which the given bundle is added to *each entity* spawned by the preset.
    ///
    /// # Examples
    ///
    /// Attaching [`Scale`](crate::prelude::Scale) modifier to every entity in the preset:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut world = World::new();
    /// world.spawn(Bindings::spawn(
    ///     Axial::right_stick().with(Scale::splat(0.1)),
    /// ));
    ///
    /// // This will be quavalent to the following:
    /// world.spawn(bindings![
    ///     (GamepadAxis::RightStickX, Scale::splat(0.1)),
    ///     (
    ///         GamepadAxis::RightStickY,
    ///         SwizzleAxis::YXZ,
    ///         Scale::splat(0.1)
    ///     )
    /// ]);
    ///
    /// #[derive(InputAction)]
    /// #[action_output(f32)]
    /// struct Movement;
    /// ```
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
    ///     Bidirectional::new(GamepadButton::DPadLeft, GamepadButton::DPadRight).with(Negate::x()),
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
    ///     Action::<Movement>::new(),
    ///     Negate::x(),
    ///     Bindings::spawn(Bidirectional::new(GamepadButton::DPadLeft, GamepadButton::DPadRight)),
    /// ));
    ///
    /// #[derive(InputAction)]
    /// #[action_output(f32)]
    /// struct Movement;
    /// ```
    fn with(self, bundle: B) -> Self::Output;
}

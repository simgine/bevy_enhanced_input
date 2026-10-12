/*!
Preset bindings for common input binding patterns.

Consider a common pattern of moving a character using keyboard keys and a gamepad stick.
With the help of [modifiers](crate::modifier) you can achieve this as follows:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, Default, Clone)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

fn spawn(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Player
        Actions<Player> [
            Action::<Movement>::new()
            // Modifier components at the action level.
            DeadZone // Apply non-uniform normalization that works for both digital and analog inputs, otherwise diagonal movement will be faster.
            SmoothNudge // Apply smoothing.
            DeltaScale // Multiply by delta time to make it framerate-independent.
            ~Scale::splat(30.0) // Additionally multiply by a constant to achieve the desired speed.
            Bindings [
                // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
                // so you need to assign keys to axes using swizzle to reorder them and negation.
                Binding(KeyCode::KeyW)
                ~SwizzleAxis::YXZ
                --
                Binding(KeyCode::KeyA)
                ~Negate::all()
                --
                Binding(KeyCode::KeyS)
                ~Negate::all()
                ~SwizzleAxis::YXZ
                --
                Binding(KeyCode::KeyD)
                --
                // In Bevy sticks split by axes and captured as 1-dimensional inputs,
                // so Y stick needs to be sweezled into Y axis.
                Binding(GamepadAxis::LeftStickX)
                --
                Binding(GamepadAxis::LeftStickY)
                ~SwizzleAxis::YXZ
            ]
        ]
    });
}
```

However, this is quite onerous! It would be inconvenient to bind WASD keys and analog sticks manually, like in the example above,
every time. You can use [`cardinal::wasd_keys`] and [`axial::left_stick`] presets to simplify the example above.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, Default, Clone)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

fn spawn(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Player
        Actions<Player> [
            Action::<Movement>::new()
            DeadZone
            SmoothNudge
            DeltaScale
            ~Scale::splat(30.0)
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
        ]
    });
}
```

# Implementation details

Each preset function defined in this module generates a list of binding entities with the appropriate components
attached to them.

To achieve this, each function returns a [`SceneList`], which you can use inside [`Bindings`](crate::prelude::Bindings).
The [`prelude`](crate::prelude) re-exports this `preset` module.

Each preset module also have a `new` function that accepts [`IntoBindingSceneBundle`]: either an input like
[`KeyCode::KeyW`], or a tuple with additional components or bundles. They must be cloneable and have no
[`bundle effects`](bevy::ecs::bundle::NoBundleEffect). Duplicate components within a bundle will panic.

# Bundles

A bundle-based API is also available. Preset structs such as [`Cardinal`](cardinal::Cardinal) and
[`Axial`](axial::Axial) implement [`SpawnableList`](bevy::ecs::spawn::SpawnableList). Their fields
are bundles, and [`WithBundle::with`] attaches components to every binding.

Similar to other [`SpawnableList`](bevy::ecs::spawn::SpawnableList)s in Bevy, like [`SpawnWith`]
or [`SpawnIter`], you need to call [`Bindings::spawn`](bevy::prelude::SpawnRelated)
from the [`SpawnRelated`] trait to generate the binding entities.

You cannot use the [`bindings!`](crate::prelude::bindings) macro.

# Examples

Adding additional bindings:

## BSN

Mix preset scene lists with individual binding scenes, separated by `--`:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
let scene = bsn! {
    Bindings [
        {preset::cardinal::wasd_keys()}
        --
        {preset::axial::left_stick()}
        --
        Binding(KeyCode::ArrowUp)
        ~SwizzleAxis::YXZ
    ]
};
```

## Bundles

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

## BSN

Pass inputs directly, or use tuples to attach modifiers to individual bindings.
To attach the same modifiers to both axes, pass them to each argument of [`axial::new`]:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
let scene = bsn! {
    Bindings [
        {preset::bidirectional::new(
            (KeyCode::NumpadAdd, Scale::splat(2.0)),
            KeyCode::NumpadSubtract,
        )}
        --
        {preset::axial::new(
            (GamepadAxis::LeftStickX, Scale::splat(1.0), Negate::all()),
            (GamepadAxis::LeftStickY, Scale::splat(1.0), Negate::all()),
        )}
    ]
};
```

As with [`WithBundle::with`], avoid duplicating modifiers already supplied by a preset.
Alternatively, attach shared modifiers at the action level, as in the movement example above.

## Bundles

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
Bindings::spawn((
    Bidirectional {
        // Struct fields are bundles, so you can also attach modifiers to individual fields.
        positive: (Binding::from(KeyCode::NumpadAdd), Scale::splat(2.0)),
        negative: Binding::from(KeyCode::NumpadSubtract),
    },
    Axial::left_stick().with((Scale::splat(1.0), Negate::all())), // Attach components to each field.
));
```

Loading from settings:

## BSN

The BSN constructors accept stored [`Binding`] values, including [`Binding::None`]:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
// Could be loaded from a file.
// `Binding::None` represents unbound inputs.
let settings = InputSettings {
    forward: [Binding::from(KeyCode::KeyW), Binding::None],
    right: [Binding::from(KeyCode::KeyD), Binding::None],
    backward: [Binding::from(KeyCode::KeyS), Binding::None],
    left: [Binding::from(KeyCode::KeyA), Binding::None],
};

let scene = bsn! {
    Bindings [
        {preset::cardinal::new(
            settings.forward[0],
            settings.right[0],
            settings.backward[0],
            settings.left[0],
        )}
        --
        {preset::cardinal::new(
            settings.forward[1],
            settings.right[1],
            settings.backward[1],
            settings.left[1],
        )}
    ]
};

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

## Bundles

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
// Could be loaded from a file.
// `Binding::None` represents unbound inputs.
let settings = InputSettings {
    forward: [Binding::from(KeyCode::KeyW), Binding::None],
    right: [Binding::from(KeyCode::KeyD), Binding::None],
    backward: [Binding::from(KeyCode::KeyS), Binding::None],
    left: [Binding::from(KeyCode::KeyA), Binding::None],
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

use bevy::{ecs::bundle::NoBundleEffect, prelude::*, scene::SceneFunction};

use crate::binding::Binding;

/// A binding value, optionally followed by additional bundles in a tuple, for a BSN preset.
///
/// Unlike the bundle API, these bundles must be cloneable and cannot
/// have bundle effects.
pub trait IntoBindingSceneBundle {
    /// Converts the input into a bundle that can be used by a scene template.
    fn into_binding_scene_bundle(self) -> impl Bundle<Effect: NoBundleEffect> + Clone;
}

impl<B: Into<Binding>> IntoBindingSceneBundle for B {
    fn into_binding_scene_bundle(self) -> impl Bundle<Effect: NoBundleEffect> + Clone {
        self.into()
    }
}

macro_rules! impl_into_binding_scene_bundle {
    ($($C:ident),*) => {
        impl<B: Into<Binding>, $($C,)*> IntoBindingSceneBundle for (B, $($C,)*)
        where
            $($C: Bundle<Effect: NoBundleEffect> + Clone,)*
        {
            #[allow(non_snake_case)]
            fn into_binding_scene_bundle(self) -> impl Bundle<Effect: NoBundleEffect> + Clone {
                let (binding, $($C,)*) = self;
                (binding.into(), $($C,)*)
            }
        }
    };
}

variadics_please::all_tuples!(impl_into_binding_scene_bundle, 0, 14, C);

fn binding_scene(bundle: impl Bundle<Effect: NoBundleEffect> + Clone) -> impl Scene {
    SceneFunction(move |_, scene| {
        scene.push_bundle_template(template(move |_| Ok(bundle.clone())));
    })
}

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

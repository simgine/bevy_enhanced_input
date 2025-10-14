/*!
Action values are stored in two forms:
- In a typed form, as the [`Action<C>`] component.
- In a dynamically typed form, as the [`ActionValue`], which is one of the required components of [`Action<C>`].
  Its variant depends on the [`InputAction::Output`].

During [`EnhancedInputSet::Update`], input is read for each [`Binding`] as an [`ActionValue`], with the variant depending
on the input source. This value is then converted into the [`ActionValue`] on the associated action entity. For example,
key inputs are captured as [`bool`], but if the action's output type is [`Vec2`], the value will be assigned to the X axis
as `0.0` or `1.0`. See [`Binding`] for details on how each source is captured, and [`ActionValue::convert`] for how values
are transformed.

Then, during [`EnhancedInputSet::Apply`], the value from [`ActionValue`] is written into [`Action<C>`].

However, you might want to apply preprocessing first - for example, invert values, apply sensitivity, or remap axes. This is
where [input modifiers](crate::modifier) come in. They are components that implement the [`InputModifier`] trait and can
be attached to both actions and bindings. Binding-level modifiers are applied first, followed by action-level modifiers.
Within a single level, modifiers are evaluated in their insertion order. Use action-level modifiers as global modifiers that
are applied to all bindings of the action.

You can see how this works by examining this expanded example, taken from the [`preset`](crate::preset) module docs:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

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

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;
```

As discussed in the [`preset`](crate::preset) module, this can be simplified substantially using presets like [`Cardinal`](crate::Cardinal) and [`Axial`](crate::Axial)!
*/

pub mod accumulate_by;
pub mod clamp;
pub mod dead_zone;
pub mod delta_scale;
pub mod exponential_curve;
pub mod fns;
pub mod linear_step;
pub mod negate;
pub mod scale;
pub mod smooth_nudge;
pub mod swizzle_axis;

use core::fmt::Debug;

use crate::prelude::*;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be attached both to bindings and actions.
///
/// If you create a custom modifier, it needs to be registered using
/// [`InputModifierAppExt::add_input_modifier`].
pub trait InputModifier: Debug {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn transform(
        &mut self,
        actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue;
}

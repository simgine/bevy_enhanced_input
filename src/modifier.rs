/*!
Action values are stored in two forms:
- In a typed form, as the [`Action<C>`] component.
- In a dynamically typed form, as the [`ActionValue`], which is one of the required components of [`Action<C>`].
  Its variant depends on the [`InputAction::Output`].

During [`EnhancedInputSet::Update`], input is read for each [`Binding`] as an [`ActionValue`], with the variant depending
on the input source. This value is then converted into the [`ActionValue`] on the associated action entity. For example,
key inputs are captured as [`bool`], but if the action's output type is [`Vec2`](bevy::math::Vec2), the value will be assigned to the X axis
as `0.0` or `1.0`. See [`Binding`] for details on how each source is captured, and [`ActionValue::convert`] for how values
are transformed.

Then, during [`EnhancedInputSet::Apply`], the value from [`ActionValue`] is written into [`Action<C>`].

Modifiers are added as components to either the binding or the action entity.
If they are attached to the action entity, they affect all bindings of the action
and are applied after all binding-level modifiers.
Within a single level, modifiers are evaluated in their insertion order.

Applying bindings at the input level allows you to have different behaviors for different input sources.
You may want to have a dead zone for analog sticks, but not for keyboard keys,
or scale sensitivity differently for mouse and gamepad inputs.

Creating common modifier configurations can be repetitive.
To simplify this, we've provided several [preset modifiers](crate::preset) that cover common use cases.

# Example

This example uses the preset modifiers to quickly create and bind a zoom action for a fly camera,
and tweaks it further using both action and input-level modifiers.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
struct FlyCam;

#[derive(InputAction)]
#[action_output(f32)]
struct Zoom;

let mut world = World::new();
world.spawn((
    Camera3d::default(),
    FlyCam,
    actions!(FlyCam[
        (
            Action::<Zoom>::new(),
            // Apply scale at the action level for all bindings.
            Scale::splat(0.1),
            Bindings::spawn((
                // In Bevy, vertical scrolling maps to the Y axis,
                // so we apply `SwizzleAxis` to map it to our 1-dimensional action.
                Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
                Bidirectional::up_down_dpad(),
            )),
        ),
    ]),
));
```
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

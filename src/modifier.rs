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

Modifiers are added as components to either the binding or the action entity.
If they are attached to the action entity, they affect all bindings of the action
and are applied after all binding-level modifiers.
Within a single level, modifiers are evaluated in their insertion order.

Applying bindings at the input level allows you to have different behaviors for different input sources.
You may want to have a dead zone for analog sticks, but not for keyboard keys,
or scale sensitivity differently for mouse and gamepad inputs.

# Example

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
struct FlyCam;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

let mut world = World::new();
world.spawn((
    Camera3d::default(),
    Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    FlyCam,
    actions!(FlyCam[
        (
            Action::<Movement>::new(),
            DeadZone::default(),
            SmoothNudge::default(),
            // This example uses the preset bindings
            Bindings::spawn((
                Axial::left_stick(),
                Cardinal::wasd_keys(),
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

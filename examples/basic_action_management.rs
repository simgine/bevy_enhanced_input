//! Demonstrates basic mapping between inputs and actions.
//!
//! This example uses a simple fly camera with a single context
//! to demonstrate how to set up actions, define keybinds and respond to actions
//! to modify the state of your application.

use core::f32::consts::{FRAC_PI_2, FRAC_PI_8};

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<FlyCam>() // All contexts should be registered.
        .add_observer(apply_movement)
        .add_observer(rotate)
        .add_observer(zoom)
        .add_observer(capture_cursor)
        .add_observer(release_cursor)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut cursor_options: Single<&mut CursorOptions>) {
    grab_cursor(&mut cursor_options, true);

    // Spawn a camera with an input context.
    commands.spawn_scene(bsn! {
        Camera3d
        ~Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y)
        FlyCam
        // Actions are related to specific context since a single entity can have multiple contexts.
        Actions<FlyCam> [
            Action::<Movement>::new()
            // Conditions and modifiers as components.
            DeadZone // Apply non-uniform normalization that works for both digital and analog inputs, otherwise diagonal movement will be faster.
            SmoothNudge // Apply smoothing.
            DeltaScale // Multiply by delta time to make it framerate-independent.
            ~Scale::splat(10.0) // Additionally multiply by a constant to achieve the desired speed.
            // Bindings are entities related to actions.
            // An action can have multiple bindings and will respond to any of them.
            Bindings [
                // Bindings like WASD or sticks are very common,
                // so we provide built-in presets to assign all keys/axes at once.
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<Rotate>::new()
            DeltaScale
            Bindings [
                // You can attach modifiers to individual bindings as well.
                Binding::mouse_motion()
                ~Negate::all()
                --
                {preset::axial::new(
                    (GamepadAxis::RightStickX, Scale::splat(100.0), Negate::x()),
                    (GamepadAxis::RightStickY, Scale::splat(100.0), Negate::x()),
                )}
            ]
            --
            Action::<Zoom>::new()
            ~Scale::splat(0.1)
            Bindings [
                // In Bevy, vertical scrolling maps to the Y axis,
                // so we apply `SwizzleAxis` to map it to our 1-dimensional action.
                Binding::mouse_wheel()
                ~SwizzleAxis::YXZ
                --
                {preset::bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown)}
            ]
            --
            Action::<CaptureCursor>::new()
            Bindings [
                Binding(MouseButton::Left)
            ]
            --
            Action::<ReleaseCursor>::new()
            Bindings [
                Binding(KeyCode::Escape)
            ]
        ]
    });

    // Setup simple 3D scene.
    commands.spawn_scene(bsn! {
        Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::splat(25.0))))
        MeshMaterial3d<StandardMaterial>(asset_value(Color::WHITE))
    });
    commands.spawn_scene(bsn! {
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        ~Transform::from_xyz(0.0, 0.5, 0.0)
    });
    commands.spawn_scene(bsn! {
        PointLight { shadow_maps_enabled: true }
        ~Transform::from_xyz(4.0, 8.0, 4.0)
    });
}

fn apply_movement(movement: On<Fire<Movement>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(movement.context).unwrap();

    // Move to the camera direction.
    let rotation = transform.rotation;

    // Movement consists of X and -Z components, so swap Y and Z with negation.
    // We could do it with modifiers, but it wold be weird for an action to return
    // a `Vec3` like this, so we doing it inside the function.
    let mut velocity = movement.value.extend(0.0).xzy();
    velocity.z = -velocity.z;

    transform.translation += rotation * velocity
}

fn rotate(
    rotate: On<Fire<Rotate>>,
    mut transforms: Query<&mut Transform>,
    cursor_options: Single<&CursorOptions>,
) {
    if cursor_options.visible {
        return;
    }

    let mut transform = transforms.get_mut(rotate.context).unwrap();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

    yaw += rotate.value.x.to_radians();
    pitch += rotate.value.y.to_radians();
    // Clamp the updated pitch to prevent gimbal locking when looking straight up or down
    pitch = pitch.clamp(-89.9f32.to_radians(), 89.9f32.to_radians());

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn zoom(zoom: On<Fire<Zoom>>, mut projections: Query<&mut Projection>) {
    let mut projection = projections.get_mut(zoom.context).unwrap();
    let Projection::Perspective(projection) = &mut *projection else {
        panic!("camera should be perspective");
    };
    projection.fov = (projection.fov + zoom.value).clamp(FRAC_PI_8, FRAC_PI_2);
}

fn capture_cursor(
    _on: On<Complete<CaptureCursor>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    grab_cursor(&mut cursor_options, true);
}

fn release_cursor(
    _on: On<Complete<ReleaseCursor>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    grab_cursor(&mut cursor_options, false);
}

fn grab_cursor(cursor_options: &mut CursorOptions, grab: bool) {
    cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    cursor_options.visible = !grab;
}

// Since it's possible to have multiple input contexts on a single entity,
// you need to define a marker component and register it in the app.
#[derive(Component, Default, Clone)]
struct FlyCam;

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only attribute is `action_output`, which defines the output type.
#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Rotate;

#[derive(InputAction)]
#[action_output(f32)]
struct Zoom;

#[derive(InputAction)]
#[action_output(bool)]
struct CaptureCursor;

#[derive(InputAction)]
#[action_output(bool)]
struct ReleaseCursor;

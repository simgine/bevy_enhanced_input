//! Simple fly camera with a single context.

use core::f32::consts::{FRAC_PI_2, FRAC_PI_8};

use bevy::{prelude::*, window::CursorGrabMode};
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Single<&mut Window>,
) {
    grab_cursor(&mut window, true);

    // Spawn a camera with an input context.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
        // Similar to `related!`, but you only specify the context type.
        // Actions are related to specific context since a single entity can have multiple contexts.
        actions!(FlyCam[
            (
                Action::<Move>::new(),
                // Conditions and modifiers as components.
                DeadZone::default(), // Apply non-uniform normalization that works for both digital and analog inputs, otherwise diagonal movement will be faster.
                SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
                Scale::splat(0.3), // Additionally multiply by a constant to achieve the desired speed.
                // Bindings are entities related to actions.
                // An action can have multiple bindings and will respond to any of them.
                Bindings::spawn((
                    // Bindings like WASD or sticks are very common,
                    // so we provide built-in `SpawnableList`s to assign all keys/axes at once.
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),
            (
                Action::<Rotate>::new(),
                Bindings::spawn((
                    // Bevy requires single entities to be wrapped in `Spawn`.
                    // You can attach modifiers to individual bindings as well.
                    Spawn((Binding::mouse_motion(), Scale::splat(0.1), Negate::all())),
                    Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                )),
            ),
            (
                Action::<Zoom>::new(),
                Scale::splat(0.1),
                Bindings::spawn((
                    // In Bevy, vertical scrolling maps to the Y axis,
                    // so we apply `SwizzleAxis` to map it to our 1-dimensional action.
                    Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ)),
                    Bidirectional::up_down_dpad(),
                )),
            ),
            // For bindings we also have a macro similar to `children!`.
            (Action::<CaptureCursor>::new(), bindings![MouseButton::Left]),
            (Action::<ReleaseCursor>::new(), bindings![KeyCode::Escape]),
        ]),
    ));

    // Setup simple 3D scene.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(25.0)))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(trigger.target()).unwrap();

    // Move to the camera direction.
    let rotation = transform.rotation;

    // Movement consists of X and -Z components, so swap Y and Z with negation.
    // We could do it with modifiers, but it wold be weird for an action to return
    // a `Vec3` like this, so we doing it inside the function.
    let mut movement = trigger.value.extend(0.0).xzy();
    movement.z = -movement.z;

    transform.translation += rotation * movement
}

fn rotate(
    trigger: Trigger<Fired<Rotate>>,
    mut transforms: Query<&mut Transform>,
    window: Single<&Window>,
) {
    if window.cursor_options.visible {
        return;
    }

    let mut transform = transforms.get_mut(trigger.target()).unwrap();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

    yaw += trigger.value.x.to_radians();
    pitch += trigger.value.y.to_radians();

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn zoom(trigger: Trigger<Fired<Zoom>>, mut projections: Query<&mut Projection>) {
    let mut projection = projections.get_mut(trigger.target()).unwrap();
    let Projection::Perspective(projection) = &mut *projection else {
        panic!("camera should be perspective");
    };
    projection.fov = (projection.fov + trigger.value).clamp(FRAC_PI_8, FRAC_PI_2);
}

fn capture_cursor(_trigger: Trigger<Completed<CaptureCursor>>, mut window: Single<&mut Window>) {
    grab_cursor(&mut window, true);
}

fn release_cursor(_trigger: Trigger<Completed<ReleaseCursor>>, mut window: Single<&mut Window>) {
    grab_cursor(&mut window, false);
}

fn grab_cursor(window: &mut Window, grab: bool) {
    window.cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    window.cursor_options.visible = !grab;
}

// Since it's possible to have multiple input contexts on a single entity,
// you need to define a marker component and register it in the app.
#[derive(Component)]
struct FlyCam;

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only attribute is `action_output`, which defines the output type.
#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

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

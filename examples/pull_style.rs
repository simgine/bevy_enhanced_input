//! Pull style as the name implies, means we are pulling the values from the source input.
//! It is used when we want to do some specific pre-processing or transform somewhat the given input value, here is a third person camera example to teach you how to do it
use core::f32::consts::PI;

use bevy::{
    color::palettes::css::SILVER,
    prelude::*,
    window::{CursorGrabMode, WindowFocused},
};
use bevy_enhanced_input::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, EnhancedInputPlugin));

    app.add_observer(release_cursor)
        .add_observer(player_movement);

    app.add_input_context::<CameraInputs>()
        .add_input_context::<Player>();

    app.add_systems(Startup, setup);

    app.add_systems(Update, (lock_cursor_on_tps, orbit_around));

    // Needed to avoid stuttering
    app.add_systems(
        PostUpdate,
        follow_anchor.before(TransformSystem::TransformPropagate),
    );

    app.run();
}

/// Marks the entity this camera will follow
#[derive(Component)]
struct AnchorMarker;

#[derive(Component)]
struct Player;

impl Player {
    fn player_context() -> impl Bundle {
        (
            Self,
            actions![
                Self[(
                Action::<Move>::new(),
                DeadZone::default(),
                Scale::splat(0.3),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
                ),]
            ],
        )
    }
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

/// A context that is added on top of [`CameraInputs`] it contains
/// ## Actions
/// [`GrabCursor`], [`ReleaseCursor`]
#[derive(Component, Reflect)]
struct CameraInputs;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct GrabCursor;

#[derive(InputAction)]
#[action_output(bool)]
struct ReleaseCursor;

impl CameraInputs {
    fn default_context() -> impl Bundle {
        (
            Self,
            actions![
                Self[
                (
                    Action::<GrabCursor>::new(),
                    SmoothNudge::new(12.),
                    Bindings::spawn(Spawn(Binding::mouse_motion()))
                ),
                (Action::<ReleaseCursor>::new(), bindings![KeyCode::Escape]),
                ]
            ],
        )
    }
}

/// Configurable infos for your camera
#[derive(Component, Reflect)]
struct ThirdPersonCameraInfos {
    /// Base radius of the camera ideally this value should never change!
    pub radius: f32,
    /// How much above the camera should be when it comes to is anchor entity
    pub y_offset: f32,
    /// While colliding with a floor/wall and so on camera should be slightly offset upwards to give a nice impression!
    pub y_offset_while_hitting: f32,
    /// Yaw limit - Limits horizontol movement
    pub yaw_limit: Option<(f32, f32)>,
    /// Pitch limit - Limits vertical movement
    pub pitch_limit: Option<(f32, f32)>,
}
impl Default for ThirdPersonCameraInfos {
    fn default() -> Self {
        Self {
            radius: 6.,
            y_offset: 1.0,
            y_offset_while_hitting: 0.35,
            yaw_limit: None,
            pitch_limit: Some((-90f32.to_radians(), 85f32.to_radians())),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraInputs::default_context(),
        ThirdPersonCameraInfos::default(),
    ));

    // Anchor entity
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        AnchorMarker,
        Player::player_context(),
        Transform::from_xyz(0.0, 0.85, 0.0),
    ));

    // Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

/// Unfurtanately this is the only way I found to guarantee that we are able to lock cursor on boot up
fn lock_cursor_on_tps(mut events: EventReader<WindowFocused>, single: Single<&mut Window>) {
    let mut window = single.into_inner();
    for _ in events.read() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

/// Orbits our camera around entity marked with [`AnchorMarker`], ideally we should only have one entity marked
fn orbit_around(
    window: Single<&Window>,
    mut q_camera: Query<(&mut Transform, &mut ThirdPersonCameraInfos)>,
    action: Single<&Action<GrabCursor>>,
) {
    // Grab camera
    let Ok((mut cam_transform, infos)) = q_camera.single_mut() else {
        return;
    };

    let delta_cursor = action.into_inner();

    let window = *window;

    let delta_x = (delta_cursor.x / window.width()) * PI;
    let delta_y = (delta_cursor.y / window.height()) * PI;

    // Retrieve current yaw and pitch
    let (yaw, pitch, _) = cam_transform.rotation.to_euler(EulerRot::YXZ);

    // Apply yaw limit if set
    let new_yaw = if let Some((min_yaw, max_yaw)) = infos.yaw_limit {
        (yaw - delta_x).clamp(min_yaw, max_yaw)
    } else {
        yaw - delta_x
    };

    // Apply pitch limit if set - Ideally should always have a max, to not go beyond the
    let new_pitch = if let Some((min_pitch, max_pitch)) = infos.pitch_limit {
        (pitch - delta_y).clamp(min_pitch, max_pitch)
    } else {
        pitch - delta_y
    };

    // Apply rotation after limit set
    cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, 0.0);

    cam_transform.translation =
        cam_transform
            .rotation
            .mul_vec3(Vec3::new(0.0, infos.y_offset_while_hitting, infos.radius));
}

/// Makes it so camera follows entity marked with [`AnchorMarker`]
fn follow_anchor(
    mut query: Query<(&mut Transform, &ThirdPersonCameraInfos)>,
    query_2: Query<Entity, With<AnchorMarker>>,
    transform: Query<&Transform, Without<ThirdPersonCameraInfos>>,
) {
    let Ok((mut cam_transform, tps)) = query.single_mut() else {
        return;
    };
    let Ok(anchor_entity) = query_2.single() else {
        return;
    };

    let Ok(anchor_transform) = transform.get(anchor_entity) else {
        return;
    };

    // Slight offset to push camera upwards
    cam_transform.translation = anchor_transform.translation
        + cam_transform.translation
        + Vec3::new(0.0, tps.y_offset, 0.0);
}

fn release_cursor(_trigger: Trigger<Started<ReleaseCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

fn player_movement(trigger: Trigger<Fired<Move>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(trigger.target()).unwrap();

    let mut movement = trigger.value.extend(0.0).xzy();
    movement.z = -movement.z;

    transform.translation += movement
}

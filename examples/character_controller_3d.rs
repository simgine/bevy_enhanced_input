//! Pull style as the name implies, means we are pulling the values from the source input.
//! It is used when we want to do some specific pre-processing or transform somewhat the given input value, here is a third person camera example to teach you how to do it
use bevy::{
    color::palettes::{css::SILVER, tailwind::BLUE_500},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_enhanced_input::prelude::*;
use core::f32::consts::PI;

fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, EnhancedInputPlugin));
    app.add_input_context::<CameraInputs>()
        .add_input_context::<Player>();
    app.add_systems(Startup, (spawn_scene, spawn_ui));
    app.add_systems(Update, (player_movement, orbit_around));
    app.add_observer(toggle_cursor);

    // Needed to avoid stuttering
    app.add_systems(
        PostUpdate,
        follow_anchor.before(TransformSystems::Propagate),
    );

    app.run();
}

/// Marks the entity this camera will follow
#[derive(Component)]
struct AnchorMarker;

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

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

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct RotateCamera;

#[derive(InputAction)]
#[action_output(bool)]
struct ToggleCursor;

/// A context that is added on top of [`CameraInputs`] it contains
/// ## Actions
/// [`GrabCursor`], [`ReleaseCursor`]
#[derive(Component, Reflect)]
#[component(on_add = CameraInputs::on_add)]
struct CameraInputs;

impl CameraInputs {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).insert((
            Self,
            actions![
                    Self[
                    (
                        Action::<RotateCamera>::new(),
                        SmoothNudge::new(12.),
                        Bindings::spawn(Spawn(Binding::mouse_motion()))
                    ),
                    (Action::<ToggleCursor>::new(), bindings![KeyCode::Escape]),
                ]
            ],
        ));
    }
}

/// Configurable infos for your camera
#[derive(Component, Reflect)]
struct ThirdPersonCamera {
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
impl Default for ThirdPersonCamera {
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

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cursor: Single<&mut CursorOptions>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // lock cursor on spawn
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraInputs,
        ThirdPersonCamera::default(),
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

    // cube for oriantation
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(2.0))),
        MeshMaterial3d(materials.add(Color::from(BLUE_500))),
        Transform::from_xyz(10.0, 5.0, 10.0),
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

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(20.0),
            align_items: AlignItems::Start,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            ..default()
        },
        children![
            (
                Node::default(),
                Text("WASD/left gamepad stick to move".to_string()),
            ),
            (
                Node::default(),
                Text("Mouse/right gamepad stick to rotate".to_string())
            ),
            (Node::default(), Text("Esc - toggle cursor".to_string())),
        ],
    ));
}

/// Orbits our camera around entity marked with [`AnchorMarker`], ideally we should only have one entity marked
fn orbit_around(
    window: Single<&Window>,
    mut q_camera: Query<(&mut Transform, &mut ThirdPersonCamera)>,
    action: Single<&Action<RotateCamera>>,
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
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera)>,
    anchor_transform: Single<&mut Transform, (Without<ThirdPersonCamera>, With<AnchorMarker>)>,
) {
    let Ok((mut cam_transform, tps)) = camera_query.single_mut() else {
        return;
    };

    // Slight offset to push camera upwards
    cam_transform.translation = anchor_transform.translation
        + cam_transform.translation
        + Vec3::new(0.0, tps.y_offset, 0.0);
}

fn toggle_cursor(_: On<Start<ToggleCursor>>, mut cursor: Single<&mut CursorOptions>) {
    if cursor.visible {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    } else {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }
}

fn player_movement(
    movement: Single<&Action<Move>>,
    camera_transform: Single<&Transform, With<ThirdPersonCamera>>,
    mut anchor_query: Query<&mut Transform, (With<AnchorMarker>, Without<ThirdPersonCamera>)>,
) {
    let input = *movement.into_inner();

    for mut anchor in anchor_query.iter_mut() {
        let input_dir = {
            let forward = camera_transform.forward();
            let forward_flat = Vec3::new(forward.x, 0.0, forward.z);
            let right = forward_flat.cross(Vec3::Y).normalize_or_zero();
            let direction = (right * input.x) + (forward_flat * input.y);
            direction.normalize_or_zero()
        };

        anchor.translation += input_dir;
    }
}

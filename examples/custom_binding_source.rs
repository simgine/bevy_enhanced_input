//! Maps trackpad [`PinchGesture`] to a [`Zoom`] action via [`Binding::Custom`]
//! and the [`CustomInputs`] resource.
//!
//! Pinch on a macOS/iOS trackpad to dolly the camera.

use bevy::{input::gestures::PinchGesture, prelude::*};
use bevy_enhanced_input::prelude::*;

const PINCH: &str = "pinch";

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<TrackpadCam>()
        .add_systems(PreUpdate, stage_pinch.before(EnhancedInputSystems::Update))
        .add_observer(on_zoom)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        TrackpadCam,
        actions!(
            TrackpadCam[(
                Action::<Zoom>::new(),
                Scale::splat(20.0),
                bindings![Binding::Custom(PINCH)],
            )]
        ),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.6, 0.9))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn stage_pinch(mut events: MessageReader<PinchGesture>, mut customs: ResMut<CustomInputs>) {
    let delta: f32 = events.read().map(|e| e.0).sum();
    customs.insert(PINCH, ActionValue::Axis1D(delta));
}

fn on_zoom(zoom: On<Fire<Zoom>>, mut cam: Single<&mut Transform, With<TrackpadCam>>) {
    let forward = cam.forward();
    cam.translation += forward * zoom.value;
}

#[derive(Component)]
struct TrackpadCam;

#[derive(InputAction)]
#[action_output(f32)]
struct Zoom;

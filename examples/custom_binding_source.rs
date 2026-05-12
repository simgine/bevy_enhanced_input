//! Binds Bevy's trackpad [`PinchGesture`] into a [`Zoom`] action through a
//! custom [`BindingSource`], without an upstream variant on [`Binding`].
//!
//! Pinch on a macOS/iOS trackpad to dolly the camera. On platforms that
//! don't emit `PinchGesture`, the camera stays put.

use bevy::{input::gestures::PinchGesture, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_binding_source::<PinchSource>()
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
        actions!(TrackpadCam[(
            Action::<Zoom>::new(),
            Scale::splat(20.0),
            bindings![(Binding::Custom, PinchSource::default())],
        )]),
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

/// Drains pinch events into every `PinchSource` before `EnhancedInputSystems::Update`.
fn stage_pinch(
    mut events: MessageReader<PinchGesture>,
    mut sources: Query<&mut PinchSource>,
) {
    let delta: f32 = events.read().map(|e| e.0).sum();
    if delta == 0.0 {
        return;
    }
    for mut source in &mut sources {
        source.staged += delta;
    }
}

#[derive(Component, Debug, Default)]
struct PinchSource {
    staged: f32,
}

impl BindingSource for PinchSource {
    fn read(&mut self, _: &ActionsQuery, _: &ContextTime) -> ActionValue {
        // Consume so each read returns a delta, not a running total.
        ActionValue::Axis1D(core::mem::take(&mut self.staged))
    }
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

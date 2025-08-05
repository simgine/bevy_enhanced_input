use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

const GROUND: Vec3 = Vec3::new(0.0, -200.0, 0.0);
const PLAYER: Vec2 = Vec2::new(50.0, 100.0);
const JUMP_VELOCITY: f32 = 300.0;
const GRAVITY: f32 = 900.0;
const MAX_SPEED: f32 = 90.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_systems(Startup, setup)
        .add_systems(Update, calculate_physics)
        .add_observer(apply_movement)
        .add_observer(apply_jump)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1200.0, 5.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.5))),
        Transform::from_translation(GROUND),
    ));

    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(PLAYER.x, PLAYER.y))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.5))),
        Transform::from_xyz(GROUND.x, GROUND.y + 100.0, GROUND.z),
        PlayerPhysics::default(),
        actions!(Player[
            (
                Action::<Move>::new(),
                DeadZone::default(),
                Scale::splat(5.0),
                SmoothNudge::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::South],
            )
        ]),
    ));
}

/// Apply horizontal movement
fn apply_movement(trigger: Trigger<Fired<Move>>, mut query: Query<&mut PlayerPhysics>) {
    let Ok(mut physics) = query.get_mut(trigger.target()) else {
        return;
    };
    physics.velocity.x = trigger.value * MAX_SPEED;
}

fn apply_jump(trigger: Trigger<Fired<Jump>>, mut query: Query<&mut PlayerPhysics>) {
    let mut physics = query.get_mut(trigger.target()).unwrap();
    if physics.is_grounded {
        physics.velocity.y = JUMP_VELOCITY;
        physics.is_grounded = false;
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct PlayerPhysics {
    velocity: Vec2,
    is_grounded: bool,
}

#[derive(Debug, InputAction)]
#[action_output(f32)]
struct Move;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Jump;

fn calculate_physics(time: Res<Time>, mut query: Query<(&mut Transform, &mut PlayerPhysics)>) {
    for (mut transform, mut physics) in query.iter_mut() {
        physics.velocity.y -= GRAVITY * time.delta_secs();
        transform.translation.y += physics.velocity.y * time.delta_secs();
        transform.translation.x += (physics.velocity.x * time.delta_secs()).clamp(-600.0, 600.0);

        let ground = GROUND.y + PLAYER.y / 2.0;
        if transform.translation.y <= ground {
            transform.translation.y = ground;
            physics.velocity.y = 0.0;
            physics.is_grounded = true;
        }
    }
}

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

const GROUND_LEVEL: f32 = -200.0;
const GROUND_WIDTH: f32 = 1200.0;
const PLAYER: Vec2 = Vec2::new(50.0, 100.0);
const JUMP_VELOCITY: f32 = 300.0;
const GRAVITY: f32 = 900.0;

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
        Mesh2d(meshes.add(Rectangle::new(GROUND_WIDTH, 5.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.5))),
        Transform::from_translation(Vec3::Y * GROUND_LEVEL),
    ));

    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(PLAYER.x, PLAYER.y))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.5))),
        Transform::from_translation(Vec3::Y * (GROUND_LEVEL + 500.0)),
        PlayerPhysics::default(),
        actions!(Player[
            (
                Action::<Move>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(450.0),
                Bindings::spawn((
                    Bidirectional::ad_keys(),
                    Bidirectional::left_right_arrow(),
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

fn apply_movement(trigger: Trigger<Fired<Move>>, mut query: Query<&mut PlayerPhysics>) {
    let mut physics = query.get_mut(trigger.target()).unwrap();
    physics.velocity.x = trigger.value;
}

fn apply_jump(trigger: Trigger<Fired<Jump>>, mut query: Query<&mut PlayerPhysics>) {
    let mut physics = query.get_mut(trigger.target()).unwrap();
    if physics.is_grounded {
        // Jump only if on the ground.
        physics.velocity.y = JUMP_VELOCITY;
        physics.is_grounded = false;
    }
}

fn calculate_physics(time: Res<Time>, mut query: Query<(&mut Transform, &mut PlayerPhysics)>) {
    for (mut transform, mut physics) in query.iter_mut() {
        physics.velocity.y -= GRAVITY * time.delta_secs();
        transform.translation.y += physics.velocity.y * time.delta_secs();
        transform.translation.x += physics.velocity.x * time.delta_secs();

        // Prevent moving off screen.
        const MAX_X: f32 = GROUND_WIDTH / 2.0 - PLAYER.x / 2.0;
        transform.translation.x = transform.translation.x.clamp(-MAX_X, MAX_X);

        // Check for ground collision.
        const GROUDED_Y: f32 = GROUND_LEVEL + PLAYER.y / 2.0;
        if transform.translation.y <= GROUDED_Y {
            transform.translation.y = GROUDED_Y;
            physics.velocity.y = 0.0;
            physics.is_grounded = true;
        }
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

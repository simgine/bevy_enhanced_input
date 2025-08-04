use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

const GROUND: Vec3 = Vec3::new(0.0, -200.0, 0.0);
const PLAYER: Vec2 = Vec2::new(50.0, 100.0);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_systems(Startup, setup)
        .add_systems(Update, physics_system)
        .add_observer(apply_movement)
        .add_observer(apply_jump)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D orthographic camera
    commands.spawn(Camera2d);

    // Spawn ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1200.0, 5.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.5))),
        Transform::from_xyz(GROUND.x, GROUND.y, GROUND.z),
    ));

    // Spawn player
    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(PLAYER.x, PLAYER.y))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.5))),
        Transform::from_xyz(GROUND.x, GROUND.y + 100.0, GROUND.z),
        PlayerPhysics {
            velocity: Vec2::ZERO,
            is_grounded: true,
        },
        actions!(
            Player[(
                Action::<Move>::new(),
                DeadZone::default(),
                Scale::splat(5.0),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            ),
            (
                Action::<Jump>::new(),
                bindings![ KeyCode::Space, GamepadButton::South ],
            )]
        ),
    ));
}

fn apply_movement(
    trigger: Trigger<Fired<Move>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerPhysics)>,
) {
    let Ok((mut transform, mut physics)) = query.get_mut(trigger.target()) else {
        return;
    };
    // Apply horizontal movement
    physics.velocity.x = trigger.value.x * 90.0; // Max speed
    transform.translation.x += physics.velocity.x * time.delta_secs();
    // Clamp to prevent moving off screen
    transform.translation.x = transform.translation.x.clamp(-600.0, 600.0);
}

fn apply_jump(trigger: Trigger<Fired<Jump>>, mut query: Query<&mut PlayerPhysics>) {
    let mut physics = query.get_mut(trigger.target()).unwrap();
    // Only jump if grounded
    if physics.is_grounded {
        physics.velocity.y = 170.0; // Jump velocity
        physics.is_grounded = false;
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerPhysics {
    velocity: Vec2,
    is_grounded: bool,
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Jump;

// System to handle physics (gravity and landing)
fn physics_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut PlayerPhysics)>) {
    for (mut transform, mut physics) in query.iter_mut() {
        // Apply gravity
        physics.velocity.y -= 500.0 * time.delta_secs();
        transform.translation.y += physics.velocity.y * time.delta_secs();

        let ground = GROUND.y + PLAYER.y / 2.0;
        // Check for ground collision
        if transform.translation.y <= ground {
            transform.translation.y = ground;
            physics.velocity.y = 0.0;
            physics.is_grounded = true;
        }
    }
}

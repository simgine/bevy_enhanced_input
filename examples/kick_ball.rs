//! Demonstrates flicking functionality for controllers.
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Press, *};

const BORDER_WIDTH: f32 = 500.0;
const STROKE_WIDTH: f32 = 5.0;
const BALL_RAD: f32 = 16.0;
const ACCELERATION: f32 = 150.0;
const KICK_IMPULSE: f32 = 1000.0;
const FRICTION: f32 = 25.0;
const KINETIC_FRICTION_COEFF: f32 = 3.0;
const BOUNCINESS: f32 = 0.8;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Ball>()
        .add_systems(Startup, setup)
        .add_systems(Update, calculate_physics)
        .add_observer(apply_roll)
        .add_observer(apply_kick)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Border
    let border_mesh = meshes.add(Rectangle::new(BORDER_WIDTH, STROKE_WIDTH));
    let border_mat = materials.add(Color::WHITE);
    // Bottom
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::Y * (-BORDER_WIDTH / 2.0)),
    ));
    // Top
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::Y * (BORDER_WIDTH / 2.0)),
    ));
    // Left
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::X * (-BORDER_WIDTH / 2.0))
            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians())),
    ));
    // Right
    commands.spawn((
        Mesh2d(border_mesh),
        MeshMaterial2d(border_mat),
        Transform::from_translation(Vec3::X * (BORDER_WIDTH / 2.0))
            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians())),
    ));

    let ball_mesh = meshes.add(Circle::new(BALL_RAD));
    let ball_mat = materials.add(Color::WHITE);
    commands.spawn((
        Ball,
        BallPhysics::default(),
        Mesh2d(ball_mesh),
        MeshMaterial2d(ball_mat),
        Transform::default(),
        Actions::<Ball>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let rolling = context
                .spawn((
                    Action::<Roll>::new(),
                    DeltaScale::default(),
                    Scale::splat(ACCELERATION),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ))
                .id();

            // For keyboard: Kick if space is pressed while rolling
            context.spawn((
                Action::<Kick>::new(),
                Chord::single(rolling),
                Press::new(1.0),
                bindings![KeyCode::Space],
            ));

            // For controller: Kick by flicking the controller stick
            context.spawn((
                Action::<Kick>::new(),
                // Replicate feel of Smash Bros., which requires the
                // controller to flick in ~2 frames (1/30 second)
                // for certain actions
                Flick::new(0.0333).with_actuation(0.9),
                Bindings::spawn(Axial::left_stick()),
            ));
        })),
    ));
}

fn apply_roll(roll: On<Fire<Roll>>, mut physics: Single<&mut BallPhysics>) {
    physics.velocity += roll.value;
}

fn apply_kick(kick: On<Fire<Kick>>, mut physics: Single<&mut BallPhysics>) {
    // Normalize the input to treat vectors that are barely along the threshold
    // the same way as a vector along the edge.
    let dir = kick.value.normalize();

    physics.velocity = dir * KICK_IMPULSE;
}

fn calculate_physics(time: Res<Time>, ball: Single<(&mut Transform, &mut BallPhysics)>) {
    let (mut transform, mut physics) = ball.into_inner();

    // Apply velocity to transform
    transform.translation += (physics.velocity * time.delta_secs()).extend(0.0);

    // Apply friction to velocity
    if physics.velocity.length_squared() > KINETIC_FRICTION_COEFF {
        let friction_dir = physics.velocity.normalize() * -1.0;
        physics.velocity += friction_dir * FRICTION * time.delta_secs();
    }

    // Check collision with walls and bounce
    let border_dist = BORDER_WIDTH / 2.0 - BALL_RAD;
    if transform.translation.x > border_dist {
        transform.translation.x = border_dist;
        physics.velocity.x *= -BOUNCINESS;
    }
    if transform.translation.x < -border_dist {
        transform.translation.x = -border_dist;
        physics.velocity.x *= -BOUNCINESS;
    }
    if transform.translation.y > border_dist {
        transform.translation.y = border_dist;
        physics.velocity.y *= -BOUNCINESS;
    }
    if transform.translation.y < -border_dist {
        transform.translation.y = -border_dist;
        physics.velocity.y *= -BOUNCINESS;
    }
}

#[derive(Component)]
struct Ball;

#[derive(Component, Default)]
struct BallPhysics {
    velocity: Vec2,
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Roll;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Kick;

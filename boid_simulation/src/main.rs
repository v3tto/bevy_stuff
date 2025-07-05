use bevy::{color::palettes::css::BLUE, prelude::*};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (boid_movement, naive_detection))
        .run();
}

#[derive(Component)]
struct Boid {
    velocity: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let triangle = meshes.add(Triangle2d::new(
        Vec2::Y * 10.0,
        Vec2::new(-5.0, -10.0),
        Vec2::new(5.0, -10.0),
    ));
    let color = Color::Srgba(Srgba::new(1.0, 0.0, 0.27, 1.0));
    let material = materials.add(color);
    let mut rng = rand::thread_rng();

    let boids = (0..32)
        .map(|_| {
            let x: f32 = rng.gen_range(-300.0..300.0);
            let y: f32 = rng.gen_range(-300.0..300.0);
            (
                Mesh2d(triangle.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(x, y, 0.0),
                Boid { velocity: 50.0 },
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(boids);
}

fn boid_movement(time: Res<Time>, mut query: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in &mut query {
        let forward = transform.rotation * Vec3::Y;
        let velocity = forward * boid.velocity * time.delta_secs();

        transform.translation += velocity;
    }
}

fn naive_detection(mut gizmos: Gizmos, query: Query<&Transform, With<Boid>>) {
    for self_transform in &query {
        for other_transfrom in &query {
            if self_transform.translation == other_transfrom.translation {
                continue;
            }
            let distance = self_transform
                .translation
                .distance(other_transfrom.translation);
            if distance < 50.0 {
                gizmos.line(
                    self_transform.translation.trunc(),
                    other_transfrom.translation.trunc(),
                    BLUE,
                );
            }
        }
        gizmos.circle_2d(self_transform.translation.truncate(), 50.0, BLUE);
    }
}

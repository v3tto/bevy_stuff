use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, boid_movement)
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
        Vec2::Y * 20.0,
        Vec2::new(-10.0, -20.0),
        Vec2::new(10.0, -20.0),
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
                Boid { velocity: 100.0 },
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

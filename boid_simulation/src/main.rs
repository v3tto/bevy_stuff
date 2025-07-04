use bevy::prelude::*;

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

    commands.spawn((
        Mesh2d(triangle),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Boid { velocity: 10.0 },
    ));
}

fn boid_movement(time: Res<Time>, mut query: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in &mut query {
        let forward = transform.rotation * Vec3::Y;
        let velocity = forward * boid.velocity * time.delta_secs();

        transform.translation += velocity;
    }
}

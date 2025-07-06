use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(DebugPlugin)
        .insert_resource(SpatialHash::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (forward_movement, update_spatial_hash))
        .run();
}

// ---------- RESOURCES ----------
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct GridKey {
    x: i32,
    y: i32,
}
#[derive(Resource, Default)]
struct SpatialHash(pub HashMap<GridKey, Vec<(Entity, Vec2)>>);

#[derive(Resource)]
struct PrintTimer(Timer);
// ---------- RESOURCES ----------

// ---------- COMPONENTS ---------
#[derive(Component)]
struct Boid;
// ---------- COMPONENTS ---------

// ----------- SYSTEMS -----------
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let triangle = meshes.add(Triangle2d::new(
        Vec2::Y * 6.0,
        Vec2::new(-3.0, -6.0),
        Vec2::new(3.0, -6.0),
    ));
    let color: Color = Color::Srgba(Srgba::new(1.0, 0.0, 0.27, 1.0));
    let material: Handle<ColorMaterial> = materials.add(color);
    let mut rng = rand::thread_rng();

    const NUM_BOIDS: u16 = 2000;
    let boids = (0..NUM_BOIDS)
        .map(|_| {
            let x: f32 = rng.gen_range(-300.0..300.0);
            let y: f32 = rng.gen_range(-300.0..300.0);
            let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let rotation = Quat::from_rotation_z(angle);
            (
                Mesh2d(triangle.clone()),
                MeshMaterial2d(material.clone()),
                Transform {
                    translation: Vec3::new(x, y, 0.0),
                    rotation,
                    ..Default::default()
                },
                Boid,
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(boids);
}

const CELL_SIZE: f32 = 100.0;
fn world_position_to_grid(translation: Vec2) -> GridKey {
    GridKey {
        x: (translation.x / CELL_SIZE).floor() as i32,
        y: (translation.y / CELL_SIZE).floor() as i32,
    }
}

fn update_spatial_hash(
    mut spatial_hash: ResMut<SpatialHash>,
    query: Query<(Entity, &Transform), With<Boid>>,
) {
    spatial_hash.0.clear();
    for (entity, transform) in &query {
        let position = transform.translation.truncate();
        let key = world_position_to_grid(position);
        spatial_hash
            .0
            .entry(key)
            .or_default()
            .push((entity, position));
    }
}

fn print_spatial_hash_contents(
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,
    spatial_hash: Res<SpatialHash>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (key, value) in &spatial_hash.0 {
            println!("{:?}: {:?}", key, value);
        }
    }
}

// fn naive_detection(mut gizmos: Gizmos, query: Query<&Transform, With<Boid>>) {
//     for self_transform in &query {
//         for other_transfrom in &query {
//             if self_transform.translation == other_transfrom.translation {
//                 continue;
//             }
//             let distance = self_transform
//                 .translation
//                 .distance(other_transfrom.translation);
//             if distance < 50.0 {
//                 gizmos.line_2d(
//                     self_transform.translation.truncate(),
//                     other_transfrom.translation.truncate(),
//                     BLUE,
//                 );
//             }
//         }
//         // gizmos.circle_2d(self_transform.translation.truncate(), 50.0, BLUE);
//     }
// }

const VELOCITY: f32 = 50.0;
fn forward_movement(time: Res<Time>, mut query: Query<&mut Transform, With<Boid>>) {
    for mut transform in &mut query {
        let forward = transform.rotation * Vec3::Y;
        let velocity = forward * VELOCITY * time.delta_secs();

        transform.translation += velocity;
    }
}
// ----------- SYSTEMS -----------

// ----------- PLUGINS -----------
pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Update, print_spatial_hash_contents);
    }    
}
// ----------- PLUGINS -----------

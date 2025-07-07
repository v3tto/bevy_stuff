use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(DebugPlugin)
        .insert_resource(SpatialHash::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                forward_movement,
                update_spatial_hash,
                flocking_behaviour,
                teleporting_edges,
                render_grid,
            ),
        )
        .run();
}

// ---------- RESOURCES ----------
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct GridKey {
    x: i32,
    y: i32,
}
#[derive(Resource, Default)]
struct SpatialHash(pub HashMap<GridKey, Vec<(Entity, Vec2, Quat)>>);

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
    let color: Color = Color::Srgba(Srgba::new(1.0, 0.0, 0.267, 1.0));
    let material: Handle<ColorMaterial> = materials.add(color);
    let mut rng = rand::thread_rng();

    const NUM_BOIDS: u16 = 500;
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

const CELL_SIZE: f32 = 50.0;
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
    for (entity, transform) in query {
        let position = transform.translation.truncate();
        let rotation = transform.rotation;
        let key = world_position_to_grid(position);
        spatial_hash
            .0
            .entry(key)
            .or_default()
            .push((entity, position, rotation));
    }
}

const VELOCITY: f32 = 50.0;
fn forward_movement(time: Res<Time>, query: Query<&mut Transform, With<Boid>>) {
    for mut transform in query {
        let forward = transform.rotation * Vec3::Y;
        let velocity = forward * VELOCITY * time.delta_secs();

        transform.translation += velocity;
    }
}

fn flocking_behaviour(spatial_hash: Res<SpatialHash>, query: Query<(Entity, &mut Transform), With<Boid>>) {
    for (entity, mut transform) in query {

        let mut boids_found: Vec<(Entity, Vec2, Quat)> = Vec::new();

        let position = transform.translation.truncate();
        let own_cell = world_position_to_grid(position);

        for current_x in -1..=1 {
            for current_y in -1..=1 {

                let neighbor_cell = GridKey {
                    x: own_cell.x + current_x,
                    y: own_cell.y + current_y,
                };

                if let Some(boids) = spatial_hash.0.get(&neighbor_cell) {
                    for boid in boids {
                        if boid.0 != entity {
                            boids_found.push(*boid);
                        }
                    }
                }
            }
        }

        let septaration_force = separation(position, &boids_found);
        // aligment();
        // cohesion();
        let steering = septaration_force;

        let target_angle = steering.y.atan2(steering.x);
        transform.rotation = Quat::from_rotation_z(target_angle);
    }
}

fn separation(position: Vec2, boids_found: &Vec<(Entity, Vec2, Quat)>) -> Vec2 {
    let mut separation_force = Vec2::ZERO;
    for neighbor_boid in boids_found {
        let neighbor_to_self = (position - neighbor_boid.1).normalize();
        separation_force += neighbor_to_self;
    }
    separation_force.normalize()
}
// fn aligment() {}
// fn cohesion() {}

const WORLD_WIDTH: f32 = 1920.0;
const WORLD_HEIGHT: f32 = 1080.0;
const WORLD_EDGE_X: f32 = WORLD_WIDTH / 2.0;
const WORLD_EDGE_Y: f32 = WORLD_HEIGHT / 2.0;
fn teleporting_edges(query: Query<&mut Transform, With<Boid>>) {
    for mut transform in query {
        let x = transform.translation.x;
        let y = transform.translation.y;
        if x > WORLD_EDGE_X {
            transform.translation.x = -WORLD_EDGE_X;
        } else if x < -WORLD_EDGE_X {
            transform.translation.x = WORLD_EDGE_X;
        }
        if y > WORLD_EDGE_Y {
            transform.translation.y = -WORLD_EDGE_Y;
        } else if y < -WORLD_EDGE_Y {
            transform.translation.y = WORLD_EDGE_Y;
        }
    }
}

fn render_grid(mut gizmos: Gizmos) {
    const GRID_COLOR: Color = Color::Srgba(Srgba::new(0.388, 0.780, 0.302, 0.3));
    let cell_count_x = (WORLD_WIDTH / CELL_SIZE).floor() as u32;
    let cell_count_y = (WORLD_HEIGHT / CELL_SIZE).floor() as u32;
    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(cell_count_x, cell_count_y),
            Vec2::new(CELL_SIZE, CELL_SIZE),
            GRID_COLOR,
        )
        .outer_edges();
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

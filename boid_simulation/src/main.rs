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
                update_spatial_hash,
                flocking_behaviour,
                (forward_movement, teleporting_edges).chain(),
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct EntityValues {
    entity: Entity,
    position: Vec2,
    rotation: Quat,
}

#[derive(Resource, Default)]
struct SpatialHash(pub HashMap<GridKey, Vec<EntityValues>>);

#[derive(Resource)]
struct PrintTimer(Timer);
// ---------- RESOURCES ----------

// ---------- COMPONENTS ---------
#[derive(Component, Debug)]
struct Boid {
    target_rotation: Quat,
    leader: bool,
}
// ---------- COMPONENTS ---------

// ----------- SYSTEMS -----------
const NUM_BOIDS: u16 = 3000;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let triangle = meshes.add(Triangle2d::new(
        Vec2::X * 6.0,
        Vec2::new(-6.0, -3.0),
        Vec2::new(-6.0, 3.0),
    ));
    let color: Color = Color::Srgba(Srgba::new(1.0, 0.0, 0.267, 1.0));
    let material: Handle<ColorMaterial> = materials.add(color);
    let mut rng = rand::thread_rng();

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
                Boid {
                    target_rotation: rotation,
                    leader: false,
                },
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(boids);

    commands.spawn((
        Mesh2d(triangle),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::new(0.0, 0.6, 0.859, 1.0)))),
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_rotation_z(5.934),
            ..Default::default()
        },
        Boid {
            target_rotation: Quat::from_rotation_z(5.934),
            leader: true,
        },
    ));
}

const CELL_SIZE: f32 = 25.0;
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

        let entity_values = EntityValues {
            entity: entity,
            position: position,
            rotation: rotation,
        };

        spatial_hash.0.entry(key).or_default().push(entity_values);
    }
}

const SEPARATION_WEIGHT: f32 = 1.5;
const ALIGNMENT_WEIGHT: f32 = 1.0;
const COHESION_WEIGHT: f32 = 1.0;
fn flocking_behaviour(
    mut gizmos: Gizmos,
    spatial_hash: Res<SpatialHash>,
    query: Query<(&mut Boid, Entity, &Transform)>,
) {
    for (mut boid, entity, transform) in query {
        let mut boids_found: Vec<EntityValues> = Vec::new();
        boids_found.reserve(5);

        let own_position = transform.translation.truncate();
        let own_cell = world_position_to_grid(own_position);

        for current_x in -1..=1 {
            for current_y in -1..=1 {
                let neighbor_cell = GridKey {
                    x: own_cell.x + current_x,
                    y: own_cell.y + current_y,
                };

                if let Some(boids) = spatial_hash.0.get(&neighbor_cell) {
                    for boid in boids {
                        if boid.entity != entity {
                            boids_found.push(*boid);
                        }
                    }
                }
            }
        }

        if !boids_found.is_empty() {
            let boids_fond_length = (boids_found.len()) as f32;

            let mut sum_neighbors_to_own = Vec2::ZERO;
            let mut sum_neighbors_direction = Vec2::ZERO;
            let mut sum_neighbors_position = Vec2::ZERO;

            for neighbor_boid in &boids_found {
                if boid.leader {
                    show_local_flockmates(&mut gizmos, own_position, neighbor_boid.position);
                }

                // SEPARATION
                let neighbor_to_own = own_position - neighbor_boid.position;
                sum_neighbors_to_own += neighbor_to_own.normalize_or_zero();

                // ALIGNMENT
                let neighbor_direction = neighbor_boid.rotation * Vec3::X;
                sum_neighbors_direction += neighbor_direction.truncate();

                // COHESION
                sum_neighbors_position += neighbor_boid.position;
            }
            boids_found.clear();

            let separation_force = sum_neighbors_to_own / boids_fond_length;
            let alignment_direction = (sum_neighbors_direction / boids_fond_length).normalize_or_zero();
            let cohesion_force = ((sum_neighbors_position / boids_fond_length) - own_position).normalize_or_zero();

            let target_direction =
                separation_force * SEPARATION_WEIGHT + 
                alignment_direction * ALIGNMENT_WEIGHT +
                cohesion_force * COHESION_WEIGHT;

            if target_direction.length_squared() > 0.0 {
                let target_angle = target_direction.y.atan2(target_direction.x);
                boid.target_rotation = Quat::from_rotation_z(target_angle);
            }
        }
    }
}

fn show_local_flockmates(gizmos: &mut Gizmos, own_position: Vec2, neighbor_boid_position: Vec2) {
    gizmos.line_2d(
        own_position,
        neighbor_boid_position,
        Color::Srgba(Srgba::new(0.0, 0.6, 0.859, 1.0)),
    );
}

const VELOCITY: f32 = 50.0;
const S: f32 = 0.01;
fn forward_movement(time: Res<Time>, query: Query<(&Boid, &mut Transform)>) {
    for (boid, mut transform) in query {
        transform.rotation = Quat::slerp(transform.rotation, boid.target_rotation, S);

        let forward = transform.rotation * Vec3::X;
        let new_position = forward * VELOCITY * time.delta_secs();

        transform.translation += new_position;
    }
}

const WORLD_WIDTH: f32 = 1900.0;
const WORLD_HEIGHT: f32 = 1000.0;
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

const GRID_COLOR: Color = Color::Srgba(Srgba::new(0.388, 0.780, 0.302, 0.3));
fn render_grid(mut gizmos: Gizmos) {
    let cell_count_x = (WORLD_WIDTH / CELL_SIZE).floor() as u32;
    let cell_count_y = (WORLD_HEIGHT / CELL_SIZE).floor() as u32;
    gizmos
        .grid_2d(
            Isometry2d::from_xy(0.0, 0.0),
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

fn print_boid_component(time: Res<Time>, mut timer: ResMut<PrintTimer>, query: Query<&Boid>) {
    if timer.0.tick(time.delta()).just_finished() {
        for boid in query {
            if boid.leader {
                println!("{:?}", boid);
            }
        }
    }
}
// ----------- SYSTEMS -----------

// ----------- PLUGINS -----------
pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Update, (print_spatial_hash_contents, print_boid_component));
    }
}
// ----------- PLUGINS -----------

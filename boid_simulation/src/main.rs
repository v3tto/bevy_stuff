use bevy::math::ops::atan2;
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
                (flocking_behaviour, forward_movement, teleporting_edges).chain(),
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
#[derive(Component)]
struct Boid {
    leader: bool,
}
// ---------- COMPONENTS ---------

// ----------- SYSTEMS -----------
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

    const NUM_BOIDS: u16 = 100;
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
                Boid { leader: false },
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
        Boid { leader: true },
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

fn flocking_behaviour(
    mut gizmos: Gizmos,
    spatial_hash: Res<SpatialHash>,
    query: Query<(&Boid, Entity, &mut Transform)>,
) {
    for (boid, entity, mut transform) in query {
        let mut boids_found: Vec<EntityValues> = Vec::new();

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
            let mut separation_force = Vec2::ZERO;
            // let mut alignment_direction = Vec2::ZERO;
            // let mut cohesion_force = Vec2::ZERO;

            for neighbor_boid in &boids_found {
                if boid.leader {
                    show_local_flockmates(&mut gizmos, own_position, neighbor_boid.position);
                }

                let neighbor_to_own = own_position - neighbor_boid.position;
                separation_force += neighbor_to_own;
            }
            boids_found.clear();

            let target_direction = separation_force;

            if boid.leader {
                show_target_direction(&mut gizmos, own_position, target_direction);
            }

            let target_angle = atan2(target_direction.y, target_direction.x);
            let target_rotation = Quat::from_rotation_z(target_angle);

            transform.rotation = target_rotation;
        }
    }
}

// fn separation() {}
// fn aligment() {}
// fn cohesion() {}

fn show_local_flockmates(gizmos: &mut Gizmos, own_position: Vec2, neighbor_boid_position: Vec2) {
    gizmos.line_2d(
        own_position,
        neighbor_boid_position,
        Color::Srgba(Srgba::new(0.0, 0.6, 0.859, 1.0)),
    );
}

fn show_target_direction(gizmos: &mut Gizmos, own_position: Vec2, target_direction: Vec2) {
    gizmos.line_2d(
        own_position,
        own_position + target_direction,
        Color::Srgba(Srgba::new(0.388, 0.78, 0.302, 1.0)),
    );
}

const VELOCITY: f32 = 10.0;
fn forward_movement(time: Res<Time>, query: Query<&mut Transform, With<Boid>>) {
    for mut transform in query {
        let forward = transform.rotation * Vec3::X;
        let position = forward * VELOCITY * time.delta_secs();

        transform.translation += position;
    }
}

const WORLD_WIDTH: f32 = 1200.0;
const WORLD_HEIGHT: f32 = 700.0;
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
    const GRID_COLOR: Color = Color::Srgba(Srgba::new(0.388, 0.780, 0.302, 0.2));
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

use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

// ---------- RESOURCES ----------
// ---------- RESOURCES ----------

// ---------- COMPONENTS ---------
// ---------- COMPONENTS ---------

// ----------- SYSTEMS -----------
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let particle_mesh = meshes.add(Circle::new(1.0));
    let color = Color::Srgba(Srgba::new(0.97, 0.46, 0.133, 1.0));
    let particle_material = materials.add(color);
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    let plasmodium = (0..1000).map(|_| {
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        let r = 50.0 * rng.r#gen::<f32>().sqrt();
        let x = r * theta.cos();
        let y = r * theta.sin();
        (
            Mesh2d(particle_mesh.clone()),
            MeshMaterial2d(particle_material.clone()),
            Transform::from_xyz(x, y, 0.0),
        )
    }).collect::<Vec<_>>();

    commands.spawn_batch(plasmodium);
}
// ----------- SYSTEMS -----------

// ----------- PLUGINS -----------
// ----------- PLUGINS -----------

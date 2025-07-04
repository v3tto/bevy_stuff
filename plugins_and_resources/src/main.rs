use bevy::prelude::*;

// globally unique data a.k.a resource
#[derive(Resource)]
struct GreetTimer(Timer);

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Vin Venture".to_string())));
    commands.spawn((Person, Name("Stuart Harold Pot".to_string())));
    commands.spawn((Person, Name("Soul Evans".to_string())));
}

// Res and ResMut provide read and write access
fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // updates the timer withe the time elapsed since the last update
    // causing the timer to finish
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Soul Evans" {
            name.0 = "Soul Eater".to_string();
            break;
        }
    }
}

pub struct HelloPlugin;
impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating))); // GreetTimer accepts a timer, in this case with duration of 2.0 seconds and in repeating mode
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) //this adds things like WindowPlugin, Time resource and the event loop
        .add_plugins(HelloPlugin)
        // .add_systems(Startup, add_people)
        // .add_systems(Update, (hello_world, (update_people, greet_people).chain()))
        .run();
}

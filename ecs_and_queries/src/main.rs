use bevy::prelude::*;

fn hello_world() {
    println!("hello world!");
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

// adds entities to the "world"
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Vin Venture".to_string())));
    commands.spawn((Person, Name("Stuart Harold Pot".to_string())));
    commands.spawn((Person, Name("Soul Evans".to_string())));
}

// this queries all entities with name and person
fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    // the for loop makes your there is one mutable reference at once
    for mut name in &mut query {
        if name.0 == "Soul Evans" {
            name.0 = "Soul Eater".to_string();
            break;
        }
    }
}

fn main() {
    App::new()
        .add_systems(Startup, add_people) // makes sure this system runs at the start
        .add_systems(Update, (hello_world, (update_people, greet_people).chain())) // chain() makes systems run in a sequence
        .run();
}

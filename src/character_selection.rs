use bevy::{prelude::*, transform::commands, asset::LoadedFolder};

use crate::{GameState, spawn_camera, components::{Player, Character}, CharacterFolder};
pub struct CharacterSelectionPlugin;

impl Plugin for CharacterSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CharacterSelection), (
                spawn_camera, 
                character_selection_ui,
            )
        )
        .add_systems(Update, ( 
                check_selected,
                select_character
                ).run_if(in_state(GameState::CharacterSelection))
            )
        .add_systems(OnExit(GameState::CharacterSelection), (
                despawn_all_with::<Node>,
                despawn_all_with::<Camera2d>,
            )
        );
    }
}

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct CharacterSquare;

#[derive(Component,Debug, PartialEq, Eq)]
struct Position {
    x: u8,
    y: u8
}

//const character_numbers: u8 = 10;
const max_columns: u8 = 4;

fn character_selection_ui(
    mut commands: Commands,
    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,
) {

    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len();
    for x in 0..character_numbers {
        let x = x as u8;
        let col = x % max_columns;
        let row = x / 4;
        let mut cmd = commands.spawn((
                Position { x: col, y: row},
                CharacterSquare,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(10.0),
                        height: Val::Percent(10.0),
                        position_type: PositionType::Relative,
                        top: Val::Px(10. * row as f32 * 4.),
                        left: Val::Px(10. * col as f32 * 4.),
                        ..default()
                    },
                    ..default()
                }
                ));

        if x == 0 {
            cmd.insert(Selected);
        }
    }

    commands.spawn((Player, Position { x: 0, y: 0}));

}


fn check_selected(
    mut commands: Commands,
    player: Query<&Position, With<Player>>,
    mut selected: Query<(Entity, &mut BackgroundColor), With<Selected>>,
    mut unselected: Query<(Entity, &Position, &mut BackgroundColor), (With<CharacterSquare>, Without<Selected>)>
) {

    for pos in &player {
        selected.iter().for_each(|x| {commands.entity(x.0).remove::<Selected>();});
        if let Some((entity, _, _)) = unselected.iter().find(|x| x.1 == pos) {
            commands.entity(entity).insert(Selected);
        }
    }

    for (_, mut bg) in &mut selected {
        bg.0 = Color::GREEN;
    }

    for (_, _, mut bg) in  &mut unselected {
        bg.0 = Color::RED;
    }

}

fn despawn_all_with<C: Component>(
    query: Query<Entity, With<C>>,
    mut commands: Commands,
)
{
    query.iter().for_each(|x| commands.entity(x).despawn());

}

fn select_character(
    mut player: Query<&mut Position, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,

    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,

) {
    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len() as u8;
    for mut pos in &mut player {
        if keyboard_input.just_pressed(KeyCode::KeyW) { 
            pos.y = (pos.y as i8 - 1).max(0) as u8 % (character_numbers / max_columns);
        }
        if keyboard_input.just_pressed(KeyCode::KeyD) { 
            pos.x = (pos.x as i8 + 1).max(0) as u8 % max_columns;
        }
        if keyboard_input.just_pressed(KeyCode::KeyA) { 
            pos.x  = (pos.x as i8 - 1).max(0) as u8 %  max_columns;
        }
        if keyboard_input.just_pressed(KeyCode::KeyS) { 
            pos.y = (pos.y as i8 + 1).max(0) as u8 % (character_numbers / max_columns);
        }
        if keyboard_input.just_pressed(KeyCode::Enter) { 
            next_state.set(GameState::Ready)
        }
    }
}

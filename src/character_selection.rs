use bevy::{prelude::*, asset::LoadedFolder};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};

use crate::{GameState, spawn_camera, components::{Player, Character, InputController}, CharacterFolder};

const MAX_COLUMNS: u8 = 4;

pub struct CharacterSelectionPlugin;

impl Plugin for CharacterSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CharacterSelection), (
                spawn_camera, 
                character_selection_ui,
            )
        )
        .add_systems(Update, ( 
                check_onhover_character,
                gamepad_input,
                gamepad_connections
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
struct Selected(pub Gamepad);


#[derive(Component, Clone, Debug)]
pub struct SelectedCharacter(pub Character);


#[derive(Component, PartialEq, Eq, Debug)]
struct CharacterSquare(u8);

impl CharacterSquare {
    fn new(value: u8) -> Self {
        Self(value)
    }

    fn value(&self) -> u8 {
        self.0
    }

    fn next(&mut self, max: u8) {
        self.0 = (self.0 + 1).clamp(0, max);
    }

    fn prev(&mut self, max: u8) {
        self.0 = self.0.checked_sub(1).unwrap_or(0).clamp(0, max);
    }

    fn up(&mut self, max: u8, char_per_row: u8) {
        self.0 = self.0.checked_sub(char_per_row).unwrap_or(0).clamp(0, max);
    }

    fn down(&mut self, max: u8, char_per_row: u8) {
        self.0 = (self.0 + char_per_row).clamp(0, max)
    }
}

fn character_selection_ui(
    mut commands: Commands,
    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,

    asset_server: Res<AssetServer>
) {

    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len();

    commands.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(50.),
                height: Val::Percent(50.),
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for x in 0..character_numbers {
                let x = x as u8;
                let col = x % MAX_COLUMNS;
                let row = x / 4;
                let character = pgs.get(x as usize).unwrap().clone();
                let character_name = &character.name;
                let texture = asset_server.load(&character.sprite_face);
                //add character name as text
                parent.spawn((
                        Name::new(character_name.clone()),
                        CharacterSquare::new(x),
                        SelectedCharacter(character.clone()),
                        NodeBundle {
                            background_color: BackgroundColor(Color::RED),
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(100.0),
                                position_type: PositionType::Relative,
                                top: Val::Px(100. * row as f32 * 4.),
                                left: Val::Px(100. * col as f32 * 4.),
                                ..default()
                            },
                            ..default()
                        },
                        UiImage {
                            texture,
                            ..default()
                        },
                    )).with_children(|p| {
                        p.spawn(TextBundle::from_section(character_name, TextStyle { 
                            font_size: 25.,
                            ..default()
                        }));
                });


            }
        });

}


fn check_onhover_character(
    player: Query<&CharacterSquare, With<Player>>,
    mut characters: Query<(Entity, &CharacterSquare, &mut BackgroundColor), Without<Player>>
) {

    for pos in &player {
        for (_, char_pos, mut bg) in &mut characters {
            if pos == char_pos {
                bg.0 = Color::GREEN;
            } else {
                bg.0 = Color::RED;
            }
        }
    }
}

fn despawn_all_with<C: Component>(
    query: Query<Entity, With<C>>,
    mut commands: Commands,
)
{
    query.iter().for_each(|x| commands.entity(x).despawn());

}

fn gamepad_input(
    mut commands: Commands,
    axes: Res<Axis<GamepadAxis>>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<GamepadButton>>,

    mut players: Query<(Entity, &InputController, &mut CharacterSquare, Option<&SelectedCharacter>), With<Player>>,
    character_squares: Query<(Entity, &CharacterSquare, &SelectedCharacter), Without<Player>>,

    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,
) {
    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len() as u8;
    let selected_n = players.iter().filter(|( _, _, _, selected)| selected.is_some()).count();
    let player_n = players.iter().count();

 
    for (entity, input_controller, mut character_square, selected) in &mut players {
        let gamepad = input_controller.0;
        let axis_lx = GamepadAxis { gamepad, axis_type: GamepadAxisType::LeftStickX };
        let axis_ly = GamepadAxis { gamepad, axis_type: GamepadAxisType::LeftStickY };
        let select_button = GamepadButton { gamepad, button_type: GamepadButtonType::East };
        let cancel_button = GamepadButton { gamepad, button_type: GamepadButtonType::South };
        let start_button = GamepadButton { gamepad, button_type: GamepadButtonType::Start };

        //if the player did not select someone
        if selected.is_none() {
            if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
                let left_stick = Vec2::new(x, y).ceil();
                if left_stick.x == 1. {
                    character_square.next(player_n as u8);
                } else if left_stick.x == -1. {
                    character_square.prev(player_n as u8);
                }

                if left_stick.y == 1. {
                    character_square.down(player_n as u8, MAX_COLUMNS);
                } else if left_stick.y == -1. {
                    character_square.up(player_n as u8, MAX_COLUMNS);
                }

            }

            if buttons.just_pressed(select_button) {
                if let Some((_, _, sel_character)) = character_squares.iter().find(|(_, char_pos, _)| char_pos.value() == character_square.value()) {
                    commands.entity(entity).insert(sel_character.clone());
                }
            }
        //if the player has already selected
        } else {
            if buttons.just_pressed(cancel_button) {
                commands.entity(entity).remove::<Selected>();
            }

            if buttons.just_pressed(start_button) && selected_n == player_n {
                next_state.set(GameState::InGame)
            }
        }

    }
}


pub fn gamepad_connections(
    mut commands: Commands,
    gamepads: Res<Gamepads>,
    mut evr_gamepad: EventReader<GamepadEvent>,
    query: Query<(Entity, &InputController)>
) {
    for ev in evr_gamepad.read() {
        // we only care about connection events
        let GamepadEvent::Connection(ev_conn) = ev else {
            continue;
        };
        match &ev_conn.connection {
            GamepadConnection::Connected(info) => {
                debug!(
                    "New gamepad connected: {:?}, name: {}",
                    ev_conn.gamepad, info.name,
                );

                commands.spawn((
                        Player, 
                        InputController(ev_conn.gamepad),
                        CharacterSquare::new(0),
                        )
                    );

            }
            GamepadConnection::Disconnected => {
                debug!("Lost connection with gamepad: {:?}", ev_conn.gamepad);
                if let Some((e, _)) = query.iter().find(|(_, x)| x.0 == ev_conn.gamepad) {
                    commands.entity(e).despawn()
                }
            }
        }
    }

    gamepads.iter()
        .filter(|gamepad| !query.iter().map(|(_, input_controller)| input_controller.0).collect::<Vec<Gamepad>>().contains(gamepad))
        .for_each(|x| {
            commands.spawn(( Player, InputController(x), CharacterSquare::new(0)));
        })

}



#[derive(Component,Debug, PartialEq, Eq)]
struct Position {
    x: u8,
    y: u8
}



fn keyboard_input(
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
            pos.y = (pos.y as i8 - 1).max(0) as u8 % (character_numbers / MAX_COLUMNS);
        }
        if keyboard_input.just_pressed(KeyCode::KeyD) { 
            pos.x = (pos.x as i8 + 1).max(0) as u8 % MAX_COLUMNS;
        }
        if keyboard_input.just_pressed(KeyCode::KeyA) { 
            pos.x  = (pos.x as i8 - 1).max(0) as u8 %  MAX_COLUMNS;
        }
        if keyboard_input.just_pressed(KeyCode::KeyS) { 
            pos.y = (pos.y as i8 + 1).max(0) as u8 % (character_numbers / MAX_COLUMNS);
        }
        if keyboard_input.just_pressed(KeyCode::Enter) { 
            next_state.set(GameState::InGame)
        }
    }
}



fn _gamepad_input(
    mut commands: Commands,
    axes: Res<Axis<GamepadAxis>>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<GamepadButton>>,

    mut query: Query<(&mut Position, &InputController), Without<CharacterSquare>>,
    character_squares: Query<(Entity, &Position), (With<CharacterSquare>, Without<Selected>)>,
    selected_characters: Query<(Entity, &Position, &Selected), With<CharacterSquare>>,

    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,
) {
    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len() as u8;
    let selected_n = selected_characters.iter().count();
    let player_n = query.iter().count();

 
    for (mut pos, input_controller) in &mut query {
        let gamepad = input_controller.0;
        let axis_lx = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickX
        };
        let axis_ly = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickY
        };
        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            let left_stick = Vec2::new(x, y).ceil();
            //da rivedere
            if left_stick != Vec2::ZERO {
                pos.y = (pos.y as f32 + left_stick.y).max(0.) as u8 % (character_numbers / MAX_COLUMNS).clamp(1, 100);
                pos.x = (pos.x as f32 + left_stick.x).max(0.) as u8 % MAX_COLUMNS;
            }
        }

        let select_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::East
        };

        let cancel_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South
        };

        let start_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::Start
        };

        if buttons.just_pressed(select_button) {
            if let Some((e, _)) = character_squares.iter().find(|(_, char_pos)| **char_pos == *pos) {
                commands.entity(e).insert(Selected(gamepad));
            }
        }

        if buttons.just_pressed(cancel_button) {
            if let Some((e, _)) = character_squares.iter().find(|(_, char_pos)| **char_pos == *pos) {
                commands.entity(e).remove::<Selected>();
            }
        }

        if buttons.just_pressed(start_button) && selected_n == player_n {
            next_state.set(GameState::InGame)
        }

    }
}

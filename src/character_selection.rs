use bevy::{prelude::*, transform::commands, asset::LoadedFolder};
use bevy::input::gamepad::{GamepadConnection, GamepadEvent};

use crate::{GameState, spawn_camera, components::{Player, Character, InputController}, CharacterFolder};
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
    commands.spawn(
        NodeBundle {
            style: Style {
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {

            for x in 0..character_numbers {
                let x = x as u8;
                let col = x % max_columns;
                let row = x / 4;
                let mut cmd = parent.spawn((
                        Position { x: col, y: row},
                        CharacterSquare,
                        NodeBundle {
                            style: Style {
                                width: Val::Px(100.0),
                                height: Val::Px(100.0),
                                position_type: PositionType::Relative,
                                top: Val::Px(100. * row as f32 * 4.),
                                left: Val::Px(100. * col as f32 * 4.),
                                ..default()
                            },
                            ..default()
                        }
                        ));

                if x == 0 {
                    cmd.insert(Selected);
                }
            }
        });

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
                // if we don't have any gamepad yet, use this one

                commands.spawn((
                        Player, 
                        InputController(ev_conn.gamepad),
                        Position { x: 0, y: 0})
                    );

            }
            GamepadConnection::Disconnected => {
                debug!("Lost connection with gamepad: {:?}", ev_conn.gamepad);
                // if it's the one we previously used for the player, remove it:
                if let Some((e, _)) = query.iter().find(|(_, x)| x.0 == ev_conn.gamepad) {
                    commands.entity(e).despawn()
                }
            }
        }
    }

    gamepads.iter()
        .filter(|gamepad|  !query.iter().map(|(_, input_controller)| input_controller.0).collect::<Vec<Gamepad>>().contains(gamepad))
        .for_each(|x| {
            commands.spawn(( Player, InputController(x), Position { x: 0, y: 0}));
        })

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

fn gamepad_input(
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    mut query: Query<(&mut Position, &InputController)>,

    characters: Res<CharacterFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loader: Res<Assets<Character>>,


) {
    let loaded_folder = loaded_folders.get(&characters.0).unwrap();
    let pgs = crate::create_characters(&loaded_folder, loader);

    let character_numbers = pgs.len() as u8;
 
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
            if left_stick != Vec2::ZERO {
                pos.y = (pos.y as f32 + left_stick.y).max(0.) as u8 % (character_numbers / max_columns).clamp(1, 100);
                pos.x = (pos.x as f32 + left_stick.x).max(0.) as u8 % max_columns;
            }
        }

        let select_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::East
        };
        if buttons.just_pressed(select_button) {
            next_state.set(GameState::Ready)
        }

    }
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

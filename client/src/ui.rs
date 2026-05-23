use bevy::prelude::*;

use crate::net_plugin::{CursorGrabState, NetworkRes};
use crate::player::PlayerInfoRes;

// ── Resources ──

#[derive(Resource)]
pub struct UiRes {
    pub last_error: Option<String>,
}

#[derive(Resource)]
pub struct PauseMenuOpen(pub bool);

// ── Components ──

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
pub struct PauseMenuUI;

#[derive(Component)]
pub struct HUDUI;

#[derive(Component)]
pub struct CoordText;

#[derive(Component)]
struct InfoText;

// ── Plugin ──

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiRes { last_error: None })
            .insert_resource(PauseMenuOpen(false))
            .add_systems(Startup, (setup_camera, setup_ground))
            .add_systems(Update, (
                ui_system,
                hud_update_system,
                button_system,
            ));
    }
}

// ── Camera & Ground (Startup) ──

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(80.0, 0.5, 80.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 0.2),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -0.25, 0.0),
        ..default()
    });
    for i in -5..=5 {
        for j in -5..=5 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.05, 0.02, 0.05)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.6, 0.3),
                    ..default()
                }),
                transform: Transform::from_xyz(i as f32 * 2.0, 0.01, j as f32 * 2.0),
                ..default()
            });
        }
    }
}

// ── UI system ──

fn ui_system(
    mut commands: Commands,
    net: Res<NetworkRes>,
    info: Res<PlayerInfoRes>,
    ui: Res<UiRes>,
    paused: Res<PauseMenuOpen>,
    menu_query: Query<Entity, With<MainMenuUI>>,
    hud_query: Query<Entity, With<HUDUI>>,
    pause_query: Query<Entity, With<PauseMenuUI>>,
) {
    if net.connected {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if hud_query.is_empty() {
            spawn_hud(&mut commands, &info);
        }
        if paused.0 && pause_query.is_empty() {
            spawn_pause_menu(&mut commands);
        }
        if !paused.0 {
            for entity in pause_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    } else if net.connecting {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in hud_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    } else {
        for entity in pause_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in hud_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if menu_query.is_empty() {
            spawn_menu(&mut commands, &ui);
        }
    }
}

// ── Menu spawners ──

fn spawn_menu(commands: &mut Commands, ui: &UiRes) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(MainMenuUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ferrite",
                TextStyle { font_size: 60.0, color: Color::WHITE, ..default() },
            ));
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(30.0), ..default() },
                ..default()
            });
            btn(parent, "Single Player");
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(10.0), ..default() },
                ..default()
            });
            btn(parent, "Multi Player");
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(10.0), ..default() },
                ..default()
            });
            btn(parent, "Quit");

            if let Some(ref err) = ui.last_error {
                parent.spawn(NodeBundle {
                    style: Style { height: Val::Px(20.0), ..default() },
                    ..default()
                });
                parent.spawn(TextBundle::from_section(
                    format!("Error: {err}"),
                    TextStyle { font_size: 16.0, color: Color::srgb(1.0, 0.0, 0.0), ..default() },
                ));
            }
        });
}

fn spawn_pause_menu(commands: &mut Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
            ..default()
        })
        .insert(PauseMenuUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Paused",
                TextStyle { font_size: 48.0, color: Color::WHITE, ..default() },
            ));
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(30.0), ..default() },
                ..default()
            });
            btn(parent, "Back to Game");
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(10.0), ..default() },
                ..default()
            });
            btn(parent, "Disconnect");
            parent.spawn(NodeBundle {
                style: Style { height: Val::Px(10.0), ..default() },
                ..default()
            });
            btn(parent, "Quit");
        });
}

fn btn(parent: &mut ChildBuilder, label: &str) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgb(0.3, 0.3, 0.3).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle { font_size: 20.0, color: Color::WHITE, ..default() },
            ));
        });
}

fn spawn_hud(commands: &mut Commands, info: &PlayerInfoRes) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .insert(HUDUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Ferrite",
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() },
            ));
            parent.spawn(TextBundle::from_section(
                "  |  ",
                TextStyle { font_size: 18.0, color: Color::srgb(0.5, 0.5, 0.5), ..default() },
            ));
            parent.spawn(TextBundle::from_section(
                "XYZ: 0.0 / 0.0 / 0.0",
                TextStyle { font_size: 18.0, color: Color::WHITE, ..default() },
            )).insert(CoordText);
            parent.spawn(TextBundle::from_section(
                format!(
                    "  |  E:{} GM:{}",
                    info.entity_id.unwrap_or(0),
                    match info.game_mode {
                        Some(0) => "S", Some(1) => "C", Some(2) => "A", Some(3) => "Sp", _ => "?",
                    }
                ),
                TextStyle { font_size: 14.0, color: Color::srgb(0.7, 0.7, 0.7), ..default() },
            )).insert(InfoText);
            parent.spawn(NodeBundle {
                style: Style { flex_grow: 1.0, ..default() },
                ..default()
            });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    margin: UiRect::right(Val::Px(6.0)),
                    ..default()
                },
                background_color: Color::srgb(0.0, 1.0, 0.0).into(),
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "Connected",
                TextStyle { font_size: 18.0, color: Color::srgb(0.0, 1.0, 0.0), ..default() },
            ));
        });
}

fn hud_update_system(
    player: Res<crate::player::PlayerRes>,
    mut query: Query<&mut Text, With<CoordText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        if let Some((x, y, z)) = player.position {
            text.sections[0].value = format!("XYZ: {:.1} / {:.1} / {:.1}", x, y, z);
        } else {
            text.sections[0].value = "XYZ: ---".to_string();
        }
    }
}

// ── Button system ──

fn button_system(
    mut pending: ResMut<crate::net_plugin::PendingConnect>,
    mut net: ResMut<NetworkRes>,
    mut paused: ResMut<PauseMenuOpen>,
    mut cursor: ResMut<CursorGrabState>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for &child in children.iter() {
                if let Ok(text) = text_query.get(child) {
                    let label = text.sections.first().map(|s| &s.value);
                    match label {
                        Some(v) if v == "Single Player" => {
                            pending.0.push(("127.0.0.1:25565".to_string(), true));
                        }
                        Some(v) if v == "Quit" => std::process::exit(0),
                        Some(v) if v == "Back to Game" => {
                            paused.0 = false;
                            cursor.want_grabbed = true;
                        }
                        Some(v) if v == "Disconnect" => {
                            net.inner = None;
                            net.connected = false;
                            net.connecting = false;
                            paused.0 = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

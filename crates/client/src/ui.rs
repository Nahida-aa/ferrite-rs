use bevy::prelude::*;

use crate::net_plugin::{CursorGrabState, NetworkRes};
use crate::player::PlayerInfoRes;

mod hud;
mod menu;
mod pause;

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
        app.insert_resource(UiRes { last_error: None })
            .insert_resource(PauseMenuOpen(false))
            .add_systems(Startup, (setup_camera, setup_ground))
            .add_systems(Update, (ui_system, hud::hud_update_system, button_system));
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
            menu::spawn_menu(&mut commands, &ui);
        }
    }
}

fn spawn_pause_menu(commands: &mut Commands) {
    pause::spawn_pause_menu(commands);
}

fn spawn_hud(commands: &mut Commands, info: &PlayerInfoRes) {
    hud::spawn_hud(commands, info);
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

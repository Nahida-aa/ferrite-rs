use bevy::prelude::*;

use crate::net_plugin::{CursorGrabState, NetworkRes};
use crate::player::PlayerInfoRes;
use crate::worlds::WorldManager;

mod hud;
mod menu;
mod pause;

// ── Resources ──

#[derive(Resource)]
pub struct UiRes {
    pub last_error: Option<String>,
}

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

#[derive(Resource)]
pub struct PauseMenuOpen(pub bool);

#[derive(Resource, Default)]
pub struct UiScreenState(pub UiScreen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScreen {
    MainMenu,
    WorldSelect,
}

impl Default for UiScreen {
    fn default() -> Self {
        Self::MainMenu
    }
}

// ── Components ──

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
struct WorldSelectUI;

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
            .init_resource::<UiScreenState>()
            .init_resource::<crate::worlds::WorldManager>()
            .init_resource::<crate::worlds::SelectedWorld>()
            .add_systems(Startup, (setup_camera, setup_ground, load_ui_font))
            .add_systems(Update, (ui_system, hud::hud_update_system, button_system));
    }
}

fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let preferred = "assets/fonts/JetBrainsMonoNerdFont.ttf";
    let asset_path = if std::path::Path::new(preferred).exists() {
        "fonts/JetBrainsMonoNerdFont.ttf"
    } else {
        // fallback bundled font
        "fonts/NotoSansSC.ttf"
    };
    commands.insert_resource(UiFont(asset_server.load(asset_path)));
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
    screen: Res<UiScreenState>,
    worlds: Res<WorldManager>,
    fonts: Res<UiFont>,
    menu_query: Query<Entity, With<MainMenuUI>>,
    world_select_query: Query<Entity, With<WorldSelectUI>>,
    hud_query: Query<Entity, With<HUDUI>>,
    pause_query: Query<Entity, With<PauseMenuUI>>,
) {
    let is_playing = net.connected;

    if is_playing {
        for entity in menu_query.iter() {
    ui_font: Res<UiFont>,
            commands.entity(entity).despawn_recursive();
        }
        if hud_query.is_empty() {
            spawn_hud(&mut commands, &info, &fonts.0);
        }
        if paused.0 && pause_query.is_empty() {
            spawn_pause_menu(&mut commands, &fonts.0);
        }
        if !paused.0 {
            for entity in pause_query.iter() {
                commands.entity(entity).despawn_recursive();
            spawn_hud(&mut commands, &info, &ui_font.0);
        }
    } else if net.connecting {
            spawn_pause_menu(&mut commands, &ui_font.0);
            commands.entity(entity).despawn_recursive();
        }
        for entity in world_select_query.iter() {
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
        match screen.0 {
            UiScreen::WorldSelect => {
                for entity in menu_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if world_select_query.is_empty() {
                    menu::spawn_world_select(&mut commands, &worlds, &fonts.0);
                }
            }
            UiScreen::MainMenu => {
                for entity in world_select_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if menu_query.is_empty() {
                    menu::spawn_menu(&mut commands, &ui, &fonts.0);
                }
            }
        }
    }
}

fn spawn_pause_menu(commands: &mut Commands, font: &Handle<Font>) {
                    menu::spawn_menu(&mut commands, &ui, &ui_font.0);
}

fn spawn_hud(commands: &mut Commands, info: &PlayerInfoRes, font: &Handle<Font>) {
    hud::spawn_hud(commands, info, font);
}

// ── Button system ──

fn button_system(
    mut pending: ResMut<crate::net_plugin::PendingConnect>,
    mut net: ResMut<NetworkRes>,
    mut paused: ResMut<PauseMenuOpen>,
    mut cursor: ResMut<CursorGrabState>,
    mut screen: ResMut<UiScreenState>,
    mut worlds: ResMut<WorldManager>,
    mut selected_world: ResMut<crate::worlds::SelectedWorld>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let label = children
                .iter()
                .find_map(|&child| text_query.get(child).ok())
                .and_then(|t| t.sections.first().map(|s| &s.value));

            let Some(label) = label else { continue; };
            let label: &str = label;
            match label {
                "Single Player" => {
                    worlds.worlds =
                        WorldManager::discover(&WorldManager::default_worlds_dir()).worlds;
                    screen.0 = UiScreen::WorldSelect;
                }
                "Demo World" => {
                    pending.0.push(("127.0.0.1:25565".to_string(), true, Some("world".to_string())));
                }
                "Quit" => std::process::exit(0),
                "Back to Game" => {
                    paused.0 = false;
                    cursor.want_grabbed = true;
                }
                "Disconnect" => {
                    net.inner = None;
                    net.connected = false;
                    net.connecting = false;
                    paused.0 = false;
                }
                "Back" => {
                    screen.0 = UiScreen::MainMenu;
                }
                "Create New World" => {
                    let name = format!("New World {}", worlds.worlds.len() + 1);
                    let dir = WorldManager::default_worlds_dir().join(&name);
                    std::fs::create_dir_all(&dir).ok();
                    selected_world.0 = Some(dir.to_string_lossy().to_string());
                    pending.0.push(("127.0.0.1:25565".to_string(), true, Some(format!("saves/{name}"))));
                }
                v => {
                    let world = worlds.worlds.iter().find(|w| w.name == *v);
                    if let Some(entry) = world {
                        selected_world.0 = Some(entry.path.to_string_lossy().to_string());
                        pending.0.push(("127.0.0.1:25565".to_string(), true, Some(format!("saves/{}", entry.name))));
                    }
                }
            }
        }
    }
}

use bevy::prelude::*;
use tokio::runtime::Runtime;

use crate::network::{Network, NetworkEvent as NetMsg};
use crate::server::ServerHandle;

// ── Resources ──

#[derive(Resource)]
pub struct NetworkRes {
    pub inner: Option<(Network, Option<ServerHandle>)>,
    pub connected: bool,
    pub connecting: bool,
}

#[derive(Resource)]
pub struct EcsRuntime(pub Runtime);

#[derive(Resource)]
pub struct PlayerRes {
    pub position: Option<(f64, f64, f64)>,
}

#[derive(Resource)]
pub struct UiRes {
    pub last_error: Option<String>,
}

// ── Plugin ──

#[derive(Component)]
struct PlayerBlock;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NetworkRes {
                inner: None,
                connected: false,
                connecting: false,
            })
            .insert_resource(PlayerRes { position: None })
            .insert_resource(UiRes { last_error: None })
            .insert_resource(EcsRuntime(Runtime::new().unwrap()))
            .init_resource::<PendingConnect>()
            .add_systems(Startup, (setup_camera, setup_grass_block))
            .add_systems(Update, (
                poll_network_system,
                handle_connections,
                camera_follow_player,
                ui_system,
                hud_update_system,
                button_system,
            ));
    }
}

// ── Camera ──

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

fn setup_grass_block(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.7, 0.3),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        PlayerBlock,
    ));
}

fn camera_follow_player(
    player: Res<PlayerRes>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    mut block_query: Query<&mut Transform, (With<PlayerBlock>, Without<Camera3d>)>,
    time: Res<Time>,
) {
    let t = time.elapsed_seconds();
    if let Some((x, y, z)) = player.position {
        let dist = 5.0;
        let cx = x as f32 + dist * t.cos();
        let cz = z as f32 + dist * t.sin();
        let cy = y as f32 + 2.0;
        if let Ok(mut cam) = query.get_single_mut() {
            cam.translation = Vec3::new(cx, cy, cz);
            cam.look_at(Vec3::new(x as f32, y as f32, z as f32), Vec3::Y);
        }
        // Move the grass block to player position
        if let Ok(mut block) = block_query.get_single_mut() {
            block.translation = Vec3::new(x as f32, y as f32, z as f32);
        }
    }
}

// ── Network system ──

fn poll_network_system(
    mut net: ResMut<NetworkRes>,
    mut player: ResMut<PlayerRes>,
    mut ui: ResMut<UiRes>,
    mut clear_color: ResMut<ClearColor>,
) {
    let event = {
        let (net_inner, _) = match &mut net.inner {
            Some(n) => n,
            None => return,
        };
        match net_inner.try_recv() {
            Ok(Some(e)) => Some(e),
            Ok(None) => None,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                tracing::info!("Network task ended");
                net.inner = None;
                net.connected = false;
                net.connecting = false;
                player.position = None;
                return;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => None,
        }
    };
    match event {
        Some(NetMsg::Connected) => {
            tracing::info!("Connected!");
            net.connecting = false;
            net.connected = true;
            clear_color.0 = Color::srgb(0.53, 0.81, 0.92);
        }
        Some(NetMsg::Disconnected(r)) => {
            tracing::info!("Disconnected: {r}");
            ui.last_error = Some(r);
            net.inner = None;
            net.connected = false;
            net.connecting = false;
            player.position = None;
            clear_color.0 = Color::srgb(0.05, 0.05, 0.05);
        }
        Some(NetMsg::PlayerPosition(x, y, z)) => {
            player.position = Some((x, y, z));
        }
        None => {}
    }
}

#[derive(Resource, Default)]
pub struct PendingConnect(pub Vec<(String, bool)>);

fn handle_connections(
    world: &mut World,
) {
    let mut pending = world.resource_mut::<PendingConnect>();
    if pending.0.is_empty() {
        return;
    }
    let connects = std::mem::take(&mut pending.0);
    let runtime_handle = world.resource::<EcsRuntime>().0.handle().clone();
    for (address, start_server) in connects {
        connect_to_server(world, &runtime_handle, &address, start_server);
    }
}

fn connect_to_server(
    world: &mut World,
    runtime_handle: &tokio::runtime::Handle,
    address: &str,
    start_server: bool,
) {
    {
        let net = world.resource::<NetworkRes>();
        if net.connecting || net.connected {
            return;
        }
    }
    {
        let mut ui = world.resource_mut::<UiRes>();
        ui.last_error = None;
    }

    let server = if start_server {
        match ServerHandle::spawn() {
            Ok(s) => {
                tracing::info!("Local server started");
                Some(s)
            }
            Err(e) => {
                tracing::error!("Start server: {e}");
                return;
            }
        }
    } else {
        None
    };

    let username = format!("FerritePlayer_{}", std::process::id());
    let (network, _join) = Network::connect(runtime_handle, address, &username);
    let mut net = world.resource_mut::<NetworkRes>();
    net.inner = Some((network, server));
    net.connecting = true;
}

// ── UI system ──

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
struct HUDUI;

#[derive(Component)]
struct CoordText;

fn ui_system(
    mut commands: Commands,
    net: Res<NetworkRes>,
    _player: Res<PlayerRes>,
    ui: Res<UiRes>,
    menu_query: Query<Entity, With<MainMenuUI>>,
    hud_query: Query<Entity, With<HUDUI>>,
) {
    if net.connected {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if hud_query.is_empty() {
            spawn_hud(&mut commands, &_player);
        }
    } else if net.connecting {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in hud_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    } else {
        for entity in hud_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if menu_query.is_empty() {
            spawn_menu(&mut commands, &ui);
        }
    }
}

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
                TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(30.0),
                    ..default()
                },
                ..default()
            });
            // Single Player button
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
                        "Single Player",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            // Multi Player button
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
                        "Multi Player",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(10.0),
                    ..default()
                },
                ..default()
            });
            // Quit button
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
                        "Quit",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });

            // Error text
            if let Some(ref err) = ui.last_error {
                parent.spawn(NodeBundle {
                    style: Style {
                        height: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });
                parent.spawn(TextBundle::from_section(
                    format!("Error: {err}"),
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(1.0, 0.0, 0.0),
                        ..default()
                    },
                ));
            }
        });
}

fn spawn_hud(commands: &mut Commands, _player: &PlayerRes) {
    // Top bar
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
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "  |  ",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.5, 0.5, 0.5),
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "XYZ: 0.0 / 0.0 / 0.0",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            )).insert(CoordText);
            parent.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "● Connected",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.0, 1.0, 0.0),
                    ..default()
                },
            ));
        });
}

fn hud_update_system(
    player: Res<PlayerRes>,
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

// ── Button interaction system ──

pub fn button_system(
    mut commands: Commands,
    mut pending: ResMut<PendingConnect>,
    interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    text_query: Query<&Text>,
) {
    for (interaction, children) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Read button text from first text child
            for &child in children.iter() {
                if let Ok(text) = text_query.get(child) {
                    if text.sections.first().map(|s| &s.value) == Some(&"Single Player".to_string()) {
                        pending.0.push(("127.0.0.1:25565".to_string(), true));
                    } else if text.sections.first().map(|s| &s.value) == Some(&"Quit".to_string()) {
                        commands.spawn(TextBundle::from_section(
                            "Quitting...",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}

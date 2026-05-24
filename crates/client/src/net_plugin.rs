use std::collections::HashMap;

use bevy::prelude::*;
use tokio::runtime::Runtime;
use ferrite_net::{Network, NetworkEvent as NetMsg};
use ferrite_gui::player::{PlayerBlock, PlayerBlockEntity, PlayerInfoRes, PlayerRes, CmdTx};
use ferrite_gui::{
    HUDUI, MainMenuUI, PauseMenuOpen, PauseMenuUI, PlayWorldButton, SelectedServer,
    ServerListUI, UiFont, UiRes, UiScreen, UiScreenState, WorldEntryButton, WorldSelectUI,
    LanDiscoveryState,
};
use ferrite_gui::worlds::{SelectedWorld, WorldManager};
use ferrite_gui::ui::server_list::{JoinServerButton, LanServerButton};

use crate::chunk_mesh::chunk_to_mesh;
use crate::server::ServerHandle;

#[derive(Resource)]
pub struct NetworkRes {
    pub inner: Option<(Network, Option<ServerHandle>)>,
    pub connected: bool,
    pub connecting: bool,
    pub net_join: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Resource)]
pub struct EcsRuntime(pub Runtime);

#[derive(Resource, Default)]
pub struct PendingConnect(pub Vec<(String, bool, Option<String>)>);

/// Tracks a server being started on a background thread.
#[derive(Resource, Default)]
pub struct PendingServerSpawn {
    pub handle: Option<std::thread::JoinHandle<anyhow::Result<ServerHandle>>>,
    pub address: Option<String>,
}

#[derive(Resource)]
pub struct CursorGrabState {
    pub want_grabbed: bool,
    pub was_grabbed: bool,
}

/// Tracks spawned chunk meshes for deduplication and cleanup.
#[derive(Resource, Default)]
pub struct ChunkEntities {
    pub entities: HashMap<(i32, i32), Entity>,
}

#[derive(Event, Debug)]
pub enum NetworkEvent {
    Connected,
    Disconnected(String),
    PlayerPosition(f64, f64, f64),
    LoginPlay {
        entity_id: i32,
        game_mode: u8,
    },
    ChunkData {
        x: i32,
        z: i32,
        chunk: ferrite_core::chunk::Chunk,
    },
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkRes {
            inner: None,
            connected: false,
            connecting: false,
            net_join: None,
        })
        .insert_resource(EcsRuntime(Runtime::new().unwrap()))
        .insert_resource(CursorGrabState {
            want_grabbed: false,
            was_grabbed: false,
        })
        .init_resource::<PendingConnect>()
        .init_resource::<PendingServerSpawn>()
        .insert_resource(ChunkEntities::default())
        .add_event::<NetworkEvent>()
        .add_systems(
            Update,
            (
                button_system,
                ferrite_gui::ui::menu::update_world_select_highlight,
                ferrite_gui::ui::menu::update_play_button_visual,
                handle_pending_connect,
                poll_server_startup,
                drain_network_events_system,
                handle_network_events_system,
                ui_system,
                lan_discovery_system,
                ferrite_gui::ui::server_list::update_server_list,
                ferrite_gui::ui::server_list::update_server_list_highlight,
                ferrite_gui::ui::server_list::update_join_button_visual,
                ferrite_gui::ui::hud::hud_update_system,
                cursor_grab_system,
            )
                .chain(),
        );
    }
}

// ── Server startup (background thread to avoid blocking the frame loop) ──

fn handle_pending_connect(
    mut pending: ResMut<PendingConnect>,
    mut server_spawn: ResMut<PendingServerSpawn>,
    mut net: ResMut<NetworkRes>,
    mut ui: ResMut<UiRes>,
) {
    if pending.0.is_empty() || net.connecting || net.connected {
        return;
    }
    if server_spawn.handle.is_some() || server_spawn.address.is_some() {
        return;
    }

    let (address, start_server, _db_path) = pending.0.remove(0);

    ui.last_error = None;

    server_spawn.address = Some(address);
    net.connecting = true;

    if start_server {
        let db_path = _db_path.unwrap_or_else(|| "world".to_string());
        tracing::info!("Starting FerrumC server in background...");
        let handle = std::thread::spawn(move || ServerHandle::spawn(&db_path));
        server_spawn.handle = Some(handle);
    }
}

fn poll_server_startup(
    mut server_spawn: ResMut<PendingServerSpawn>,
    mut cmd_tx: ResMut<CmdTx>,
    mut net: ResMut<NetworkRes>,
    mut ui: ResMut<UiRes>,
    runtime: Res<EcsRuntime>,
) {
    if server_spawn.address.is_none() {
        return;
    }

    // If a server is being started, check if it's done
    if let Some(handle) = server_spawn.handle.as_ref() {
        if !handle.is_finished() {
            return;
        }

        let handle = server_spawn.handle.take().unwrap();
        let server = match handle.join() {
            Ok(Ok(s)) => {
                tracing::info!("Local server started");
                Some(s)
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to start server: {e}");
                ui.last_error = Some(e.to_string());
                server_spawn.address = None;
                net.connecting = false;
                return;
            }
            Err(_) => {
                tracing::error!("Server thread panicked");
                server_spawn.address = None;
                net.connecting = false;
                return;
            }
        };

        let server_addr = server_spawn.address.take().unwrap();
        let username = {
            let raw = format!("f_{}", std::process::id());
            if raw.len() > 16 { raw[..16].to_string() } else { raw }
        };
        let runtime_handle = runtime.0.handle().clone();
        let (network, join) = Network::connect(&runtime_handle, &server_addr, &username);
        cmd_tx.0 = Some(network.command_sender());
        net.inner = Some((network, server));
        net.net_join = Some(join);
    } else {
        // No server needed — connect directly
        let address = server_spawn.address.take().unwrap();
        let username = {
            let raw = format!("f_{}", std::process::id());
            if raw.len() > 16 { raw[..16].to_string() } else { raw }
        };
        let runtime_handle = runtime.0.handle().clone();
        let (network, join) = Network::connect(&runtime_handle, &address, &username);
        cmd_tx.0 = Some(network.command_sender());
        net.inner = Some((network, None));
        net.net_join = Some(join);
    }
}

// ── Network event drain ──

fn drain_network_events_system(
    mut net: ResMut<NetworkRes>,
    mut network_events: EventWriter<NetworkEvent>,
) {
    let Some((net_inner, _)) = net.inner.as_mut() else {
        return;
    };

    loop {
        match net_inner.try_recv() {
            Ok(Some(NetMsg::Connected)) => {
                network_events.send(NetworkEvent::Connected);
            }
            Ok(Some(NetMsg::Disconnected(reason))) => {
                network_events.send(NetworkEvent::Disconnected(reason));
                break;
            }
            Ok(Some(NetMsg::PlayerPosition(x, y, z))) => {
                network_events.send(NetworkEvent::PlayerPosition(x, y, z));
            }
            Ok(Some(NetMsg::LoginPlay {
                entity_id,
                game_mode,
            })) => {
                network_events.send(NetworkEvent::LoginPlay {
                    entity_id,
                    game_mode,
                });
            }
            Ok(Some(NetMsg::ChunkData { x, z, chunk })) => {
                network_events.send(NetworkEvent::ChunkData { x, z, chunk });
            }
            Ok(None) => break,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                tracing::info!("Network task ended");
                network_events
                    .send(NetworkEvent::Disconnected("Network task ended".to_string()));
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
        }
    }
}

// ── Network event handling ──

fn handle_network_events_system(
    mut events: EventReader<NetworkEvent>,
    mut player: ResMut<PlayerRes>,
    mut ui: ResMut<UiRes>,
    mut clear_color: ResMut<ClearColor>,
    mut cmd_tx: ResMut<CmdTx>,
    mut block: ResMut<PlayerBlockEntity>,
    mut info: ResMut<PlayerInfoRes>,
    mut cursor: ResMut<CursorGrabState>,
    mut net: ResMut<NetworkRes>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut paused: ResMut<PauseMenuOpen>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::Connected => {
                tracing::info!("Connected!");
                net.connecting = false;
                net.connected = true;
                clear_color.0 = Color::srgb(0.53, 0.81, 0.92);
                if block.0.is_none() {
                    let e = commands
                        .spawn((
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
                        ))
                        .id();
                    block.0 = Some(e);
                }
            }
            NetworkEvent::Disconnected(reason) => {
                tracing::info!("Disconnected: {reason}");
                ui.last_error = Some(reason.clone());
                if let Some(e) = block.0.take() {
                    commands.entity(e).despawn();
                }

                // Abort the network task
                if let Some(join) = net.net_join.take() {
                    join.abort();
                }
                // Despawn all chunk meshes
                for (_, entity) in chunk_entities.entities.drain() {
                    commands.entity(entity).despawn();
                }

                net.inner = None;
                net.connected = false;
                net.connecting = false;
                paused.0 = false;
                player.position = None;
                info.entity_id = None;
                info.game_mode = None;
                cursor.want_grabbed = false;
                clear_color.0 = Color::srgb(0.05, 0.05, 0.05);
                cmd_tx.0 = None;
            }
            NetworkEvent::PlayerPosition(x, y, z) => {
                player.position = Some((*x, *y, *z));
            }
            NetworkEvent::LoginPlay {
                entity_id,
                game_mode,
            } => {
                tracing::info!("PlayerInfo: entity {} game mode {}", entity_id, game_mode);
                info.entity_id = Some(*entity_id);
                info.game_mode = Some(*game_mode);
            }
            NetworkEvent::ChunkData { x, z, chunk } => {
                tracing::info!("Building merged mesh for chunk at ({},{})", x, z);
                let mesh = chunk_to_mesh(chunk, *x, *z);
                let handle = meshes.add(mesh);
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                });
                let entity = commands.spawn(PbrBundle {
                    mesh: handle,
                    material,
                    transform: Transform::IDENTITY,
                    ..default()
                }).id();

                // Dedup: despawn old mesh for the same chunk
                if let Some(old) = chunk_entities.entities.insert((*x, *z), entity) {
                    commands.entity(old).despawn();
                }
            }
        }
    }
}

// ── Cursor grab ──

fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor: ResMut<CursorGrabState>,
    mut paused: ResMut<PauseMenuOpen>,
    net: Res<NetworkRes>,
) {
    let mut window = match windows.get_single_mut() {
        Ok(w) => w,
        Err(_) => return,
    };

    let grabbed = window.cursor.grab_mode != bevy::window::CursorGrabMode::None;

    if keyboard.just_pressed(KeyCode::Escape) && net.connected {
        paused.0 = !paused.0;
        if paused.0 {
            cursor.want_grabbed = false;
        }
    }
    if cursor.want_grabbed && cursor.was_grabbed && !grabbed {
        cursor.want_grabbed = false;
        cursor.was_grabbed = false;
        window.cursor.visible = true;
    }
    if cursor.want_grabbed && !grabbed && !paused.0 {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
        cursor.was_grabbed = true;
    }
    if (!cursor.want_grabbed || paused.0) && grabbed {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
        cursor.was_grabbed = false;
    }
    if mouse.just_pressed(MouseButton::Left) && !cursor.want_grabbed && !paused.0 && net.connected {
        cursor.want_grabbed = true;
    }
}

// ── Button system ──

fn button_system(
    mut pending: ResMut<PendingConnect>,
    mut net: ResMut<NetworkRes>,
    mut paused: ResMut<PauseMenuOpen>,
    mut cursor: ResMut<CursorGrabState>,
    mut screen: ResMut<UiScreenState>,
    mut worlds: ResMut<WorldManager>,
    mut selected_world: ResMut<SelectedWorld>,
    mut selected_server: ResMut<SelectedServer>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut commands: Commands,
    interaction_query: Query<
        (Entity, &Interaction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    lan_button_query: Query<&LanServerButton>,
    join_server_query: Query<&JoinServerButton>,
    world_entry_query: Query<&WorldEntryButton>,
    play_world_query: Query<&PlayWorldButton>,
    text_query: Query<&Text>,
) {
    for (entity, interaction, children) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Check if this is a LAN server button (click to select)
        if let Ok(btn) = lan_button_query.get(entity) {
            selected_server.0 = Some(btn.0.clone());
            continue;
        }

        // Check if this is the Join Server button
        if join_server_query.get(entity).is_ok() {
            if let Some(ref address) = selected_server.0 {
                pending.0.push((
                    address.clone(),
                    false,
                    None,
                ));
            }
            continue;
        }

        // Check if this is a world entry (click to select)
        if let Ok(entry) = world_entry_query.get(entity) {
            let name = &entry.0;
            if let Some(world) = worlds.worlds.iter().find(|w| w.name == *name) {
                selected_world.0 = Some(world.path.to_string_lossy().to_string());
            }
            continue;
        }

        // Check if this is the Play World button
        if play_world_query.get(entity).is_ok() {
            if let Some(ref path) = selected_world.0 {
                let name = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("world");
                pending.0.push((
                    "127.0.0.1:25565".to_string(),
                    true,
                    Some(format!("saves/{name}")),
                ));
            }
            continue;
        }

        let Some(label) = children
            .iter()
            .find_map(|&child| text_query.get(child).ok())
            .and_then(|t| t.sections.first().map(|s| &s.value))
        else {
            continue;
        };
        let label: &str = label;
        match label {
            "Single Player" => {
                worlds.worlds = WorldManager::discover(&WorldManager::default_worlds_dir()).worlds;
                selected_world.0 = None;
                screen.0 = UiScreen::WorldSelect;
            }
            "Demo World" => {
                pending.0.push((
                    "127.0.0.1:25565".to_string(),
                    true,
                    Some("world".to_string()),
                ));
            }
            "Quit" => std::process::exit(0),
            "Back to Game" => {
                paused.0 = false;
                cursor.want_grabbed = true;
            }
            "Disconnect" => {
                // Abort the network task
                if let Some(join) = net.net_join.take() {
                    join.abort();
                }
                // Despawn all chunk meshes
                for (_, entity) in chunk_entities.entities.drain() {
                    commands.entity(entity).despawn();
                }
                net.inner = None;
                net.connected = false;
                net.connecting = false;
                paused.0 = false;
            }
            "Multi Player" => {
                screen.0 = UiScreen::ServerList;
            }
            "Back" => {
                selected_world.0 = None;
                screen.0 = UiScreen::MainMenu;
            }
            "Create New World" => {
                let name = format!("New World {}", worlds.worlds.len() + 1);
                let dir = WorldManager::default_worlds_dir().join(&name);
                std::fs::create_dir_all(&dir).ok();
                selected_world.0 = Some(dir.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
}

fn lan_discovery_system(
    screen: Res<UiScreenState>,
    mut lan: ResMut<LanDiscoveryState>,
    net: Res<NetworkRes>,
) {
    // Initialize once at first run
    if !lan.initialized {
        lan.lan.init();
        lan.initialized = true;
    }

    let active = screen.0 == UiScreen::ServerList && !net.connected;

    if active {
        let new_servers = lan.lan.take_servers();
        if !new_servers.is_empty() {
            lan.servers = new_servers;
            lan.generation += 1;
        }
    } else if !lan.servers.is_empty() {
        lan.servers.clear();
        lan.generation += 1;
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
    server_list_query: Query<Entity, With<ServerListUI>>,
    hud_query: Query<Entity, With<HUDUI>>,
    pause_query: Query<Entity, With<PauseMenuUI>>,
) {
    let is_playing = net.connected;

    if is_playing {
        for entity in menu_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in world_select_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in server_list_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if hud_query.is_empty() {
            ferrite_gui::ui::hud::spawn_hud(&mut commands, &info, &fonts.0);
        }
        if paused.0 && pause_query.is_empty() {
            ferrite_gui::ui::pause::spawn_pause_menu(&mut commands, &fonts.0);
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
        for entity in world_select_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in server_list_query.iter() {
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
                for entity in server_list_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if world_select_query.is_empty() {
                    ferrite_gui::ui::menu::spawn_world_select(&mut commands, &worlds, &fonts.0);
                }
            }
            UiScreen::ServerList => {
                for entity in menu_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in world_select_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if server_list_query.is_empty() {
                    ferrite_gui::ui::server_list::spawn_server_list(&mut commands, &fonts.0);
                }
            }
            UiScreen::MainMenu => {
                for entity in world_select_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for entity in server_list_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                if menu_query.is_empty() {
                    ferrite_gui::ui::menu::spawn_menu(&mut commands, &ui, &fonts.0);
                }
            }
        }
    }
}

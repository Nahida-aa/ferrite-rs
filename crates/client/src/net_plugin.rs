use bevy::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use ferrite_net::{Network, NetworkCommand, NetworkEvent as NetMsg};

use crate::player::{PlayerBlock, PlayerBlockEntity, PlayerInfoRes, PlayerRes};
use crate::server::ServerHandle;
use crate::ui::{PauseMenuOpen, UiRes};

#[derive(Resource)]
pub struct NetworkRes {
    pub inner: Option<(Network, Option<ServerHandle>)>,
    pub connected: bool,
    pub connecting: bool,
}

#[derive(Resource)]
pub struct EcsRuntime(pub Runtime);

#[derive(Resource)]
pub struct CmdTx(pub Option<mpsc::Sender<NetworkCommand>>);

#[derive(Resource, Default)]
pub struct PendingConnect(pub Vec<(String, bool)>);

#[derive(Resource)]
pub struct CursorGrabState {
    pub want_grabbed: bool,
    pub was_grabbed: bool,
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
        })
        .insert_resource(EcsRuntime(Runtime::new().unwrap()))
        .insert_resource(CmdTx(None))
        .insert_resource(CursorGrabState {
            want_grabbed: false,
            was_grabbed: false,
        })
        .init_resource::<PendingConnect>()
        .add_event::<NetworkEvent>()
        .add_systems(
            Update,
            (drain_network_events_system, handle_network_events_system).chain(),
        )
        .add_systems(Update, (handle_connections, cursor_grab_system));
    }
}

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
                network_events.send(NetworkEvent::Disconnected("Network task ended".to_string()));
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
        }
    }
}

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
                net.inner = None;
                net.connected = false;
                net.connecting = false;
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
                tracing::info!("Spawning chunk at ({},{})", x, z);
                // simple spawn: for each column spawn a single cube at the highest non-air block
                let base_x = (*x as f32) * 16.0;
                let base_z = (*z as f32) * 16.0;
                for cx in 0..16usize {
                    for cz in 0..16usize {
                        // find highest non-air block
                        let mut found_y: Option<i32> = None;
                        for (si, section) in chunk.sections.iter().enumerate().rev() {
                            for local_y in (0..ferrite_core::chunk::SECTION_HEIGHT).rev() {
                                let idx = local_y
                                    * ferrite_core::chunk::CHUNK_WIDTH
                                    * ferrite_core::chunk::CHUNK_WIDTH
                                    + cz * ferrite_core::chunk::CHUNK_WIDTH
                                    + cx;
                                if let Some(bs) = section.blocks.get(idx) {
                                    // treat 0 as air
                                    let is_air = *bs == ferrite_core::block::BlockState::AIR;
                                    if !is_air {
                                        let y = si as i32
                                            * ferrite_core::chunk::SECTION_HEIGHT as i32
                                            + local_y as i32;
                                        found_y = Some(y);
                                        break;
                                    }
                                }
                            }
                            if found_y.is_some() {
                                break;
                            }
                        }
                        if let Some(y) = found_y {
                            let world_x = base_x + cx as f32 + 0.5;
                            let world_y = y as f32 + 0.5;
                            let world_z = base_z + cz as f32 + 0.5;
                            commands.spawn(PbrBundle {
                                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::srgb(0.6, 0.4, 0.2),
                                    ..default()
                                }),
                                transform: Transform::from_xyz(world_x, world_y, world_z),
                                ..default()
                            });
                        }
                    }
                }
            }
        }
    }
}

fn handle_connections(world: &mut World) {
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
    let mut cmd_tx = world.resource_mut::<CmdTx>();
    cmd_tx.0 = Some(network.command_sender());
    let mut net = world.resource_mut::<NetworkRes>();
    net.inner = Some((network, server));
    net.connecting = true;
}

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
    if mouse.just_pressed(MouseButton::Left) && !cursor.want_grabbed && !paused.0 {
        cursor.want_grabbed = true;
    }
}

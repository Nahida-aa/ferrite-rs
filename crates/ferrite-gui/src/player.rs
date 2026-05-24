use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use tokio::sync::mpsc;

pub use crate::{PauseMenuOpen, PlayerBlock, PlayerInfoRes, PlayerLookRes, PlayerRes, PlayerBlockEntity};
use ferrite_net::NetworkCommand;

#[derive(Resource)]
pub struct CmdTx(pub Option<mpsc::Sender<NetworkCommand>>);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerRes { position: None })
            .insert_resource(PlayerInfoRes::default())
            .insert_resource(PlayerLookRes::default())
            .insert_resource(crate::PlayerBlockEntity(None))
            .insert_resource(CmdTx(None))
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (look_system, camera_follow_player, movement_system));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        tonemapping: Tonemapping::None,
        ..default()
    });
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn look_system(
    mut mouse_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut look: ResMut<PlayerLookRes>,
    paused: Res<PauseMenuOpen>,
) {
    if paused.0 {
        return;
    }
    for ev in mouse_events.read() {
        look.yaw += ev.delta.x * 0.005;
        look.pitch = (look.pitch + ev.delta.y * 0.005).clamp(-1.5, 1.5);
    }
}

fn camera_follow_player(
    player: Res<PlayerRes>,
    look: Res<PlayerLookRes>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    mut block_query: Query<&mut Transform, (With<PlayerBlock>, Without<Camera3d>)>,
) {
    if let Some((x, y, z)) = player.position {
        let pos = Vec3::new(x as f32, y as f32, z as f32);
        let behind = Vec3::new(look.yaw.sin(), 0.0, -look.yaw.cos());
        let up = Vec3::new(0.0, pitch_offset(look.pitch), 0.0);
        let dist = 4.0;
        if let Ok(mut cam) = query.get_single_mut() {
            cam.translation = pos + behind * dist + up;
            cam.look_at(pos + Vec3::new(0.0, 0.5, 0.0), Vec3::Y);
        }
        if let Ok(mut block) = block_query.get_single_mut() {
            block.translation = pos;
        }
    }
}

fn pitch_offset(pitch: f32) -> f32 {
    2.0 + pitch * 1.0
}

fn movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    look: Res<PlayerLookRes>,
    mut player: ResMut<PlayerRes>,
    cmd_tx: Res<CmdTx>,
    paused: Res<PauseMenuOpen>,
) {
    if paused.0 {
        return;
    }
    let (x, y, z) = match player.position {
        Some(p) => p,
        None => return,
    };

    let forward = Vec3::new(-look.yaw.sin(), 0.0, look.yaw.cos());
    let right = Vec3::new(-look.yaw.cos(), 0.0, -look.yaw.sin());

    let mut delta = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        delta += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        delta -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        delta -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        delta += right;
    }

    if delta == Vec3::ZERO {
        return;
    }

    let speed = 0.5;
    let delta = delta.normalize() * speed;
    let new_x = x + delta.x as f64;
    let new_z = z + delta.z as f64;

    player.position = Some((new_x, y, new_z));

    if let Some(sender) = &cmd_tx.0 {
        let _ = sender.try_send(NetworkCommand::SetPosition {
            x: new_x,
            y,
            z: new_z,
            yaw: look.yaw.to_degrees(),
            pitch: look.pitch.to_degrees(),
            on_ground: true,
        });
    }
}

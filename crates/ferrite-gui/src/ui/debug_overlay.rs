use bevy::prelude::*;
use bevy_diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use crate::{
    DebugOverlayUI, DebugOverlayVisible, PlayerInfoRes, PlayerLookRes, PlayerRes,
};

const VERSION: &str = "1.21.8";
const RUST_VERSION: &str = env!("CARGO_PKG_RUST_VERSION");

pub fn spawn_debug_overlay(commands: &mut Commands, font: &Handle<Font>) {
    let lines = generate_lines(0.0, None, None, None, None, None, None, None);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            ..default()
        })
        .insert(DebugOverlayUI)
        .with_children(|parent| {
            for line in lines {
                parent.spawn(TextBundle::from_section(
                    line,
                    TextStyle {
                        font: font.clone(),
                        font_size: 12.0,
                        color: Color::srgb(0.85, 0.85, 0.85),
                    },
                ));
            }
        });
}

fn generate_lines(
    fps: f64,
    player_pos: Option<(f64, f64, f64)>,
    info: Option<&PlayerInfoRes>,
    look: Option<&PlayerLookRes>,
    chunk_count: Option<usize>,
    display_res: Option<&str>,
    server_name: Option<&str>,
    biome: Option<&str>,
) -> Vec<String> {
    let fps_str = format!("{:.0}", fps);

    let (xyz, block, chunk, facing) = match player_pos {
        Some((x, y, z)) => {
            let bx = x.floor() as i32;
            let by = y.floor() as i32;
            let bz = z.floor() as i32;
            let cx = bx >> 4;
            let cz = bz >> 4;
            let rx = if cx >= 0 { cx / 32 } else { (cx + 1) / 32 - 1 };
            let rz = if cz >= 0 { cz / 32 } else { (cz + 1) / 32 - 1 };
            (
                format!("{x:.3} / {y:.5} / {z:.3}"),
                format!("{bx} {by} {bz}"),
                format!("{cx} {cz} [{cx} {cz} in r.{rx}.{rz}.mca]"),
                facing_from_yaw(look.map(|l| l.yaw).unwrap_or(0.0)),
            )
        }
        None => (
            "TODO / TODO / TODO".to_string(),
            "TODO".to_string(),
            "TODO".to_string(),
            "TODO (TODO / TODO)".to_string(),
        ),
    };

    let entity_str = info
        .and_then(|i| i.entity_id)
        .map(|e| format!("{e}"))
        .unwrap_or_else(|| "??".to_string());
    let yaw_pitch = look
        .map(|l| format!("{:.0} / {:.1}", l.yaw.to_degrees(), l.pitch.to_degrees()))
        .unwrap_or_else(|| "? / ?".to_string());

    let chunks_str = chunk_count
        .map(|c| format!("{c}"))
        .unwrap_or_else(|| "?".to_string());

    let display_str = display_res.unwrap_or("TODO");
    let server = server_name.unwrap_or("TODO");
    let biome_str = biome.unwrap_or("TODO");

    vec![
        format!("~Ferrite {VERSION} ({VERSION}/vanilla)"),
        format!("rust: {RUST_VERSION}"),
        format!("{fps_str} fps T: TODO vsync TODO GPU: TODO%                               Mem: TODO% TODO/TODOMB"),
        format!("{server} server, TODO tx, TODO rx                                     Allocation rate: TODO MB/s"),
        "TODO C: TODO/D: TODO pC: TODO pU: TODO aB: TODO                             Allocation: TODO% TODOMB".to_string(),
        format!("E: {entity_str}/TODO, SD: TODO"),
        "TODO P: TODO T: TODO                                     CPU: TODO".to_string(),
        format!("Chunks[C] W: {chunks_str}, TODO E: TODO,TODO,TODO"),
        format!("{biome_str} FC: 0                                                Display: {display_str}"),
        "                    TODO GPU".to_string(),
        format!("XYZ: {xyz}                    TODO OpenGL"),
        format!("Block: {block}"),
        format!("Chunk: {chunk}"),
        format!("Facing: {facing} ({yaw_pitch})"),
        "Client Light: TODO (TODO sky, TODO block)".to_string(),
        "CH S: TODO M: TODO ML: TODO".to_string(),
        "SH S: ?? O: ?? M: ?? ML: ??".to_string(),
        "Biome: TODO".to_string(),
        "Local Difficulty ??".to_string(),
        "Sounds: TODO/TODO + TODO/TODO (Mood TODO%)".to_string(),
        String::new(),
        "Debug charts: [F3+1] Profiler hidden; [F3+2] FPS hidden; [F3+3] Bandwidth + Ping hidden".to_string(),
        "For help: press F3 + Q".to_string(),
    ]
}

fn facing_from_yaw(yaw_rad: f32) -> String {
    let deg = ((yaw_rad.to_degrees() % 360.0) + 360.0) % 360.0;
    let (dir, axis) = match deg as u32 {
        0..=22 | 338..=360 => ("South", "Towards positive Z"),
        23..=67 => ("Southwest", "Towards positive Z / negative X"),
        68..=112 => ("West", "Towards negative X"),
        113..=157 => ("Northwest", "Towards negative X / negative Z"),
        158..=202 => ("North", "Towards negative Z"),
        203..=247 => ("Northeast", "Towards negative Z / positive X"),
        248..=292 => ("East", "Towards positive X"),
        293..=337 => ("Southeast", "Towards positive X / positive Z"),
        _ => ("Unknown", "Unknown"),
    };
    format!("{dir} ({axis})")
}

pub fn debug_overlay_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<DebugOverlayVisible>,
    net: Query<(), With<crate::HUDUI>>,
) {
    if net.is_empty() {
        return;
    }
    if keyboard.just_pressed(KeyCode::F3) {
        visible.0 = !visible.0;
    }
}

pub fn debug_overlay_update_system(
    mut root_query: Query<&mut Children, With<DebugOverlayUI>>,
    mut text_query: Query<&mut Text>,
    diagnostics: Res<DiagnosticsStore>,
    player: Res<PlayerRes>,
    info: Res<PlayerInfoRes>,
    look: Res<PlayerLookRes>,
    windows: Query<&Window>,
    chunk_count: Res<crate::ChunkCount>,
) {
    let Ok(children) = root_query.get_single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let display_res = windows
        .iter()
        .next()
        .map(|w| format!("{}x{} ({})", w.width() as u32, w.height() as u32, "TODO"));

    let lines = generate_lines(
        fps,
        player.position,
        Some(&*info),
        Some(&*look),
        Some(chunk_count.0),
        display_res.as_deref(),
        None,
        None,
    );

    for (i, text_line) in lines.iter().enumerate() {
        if let Some(&child) = children.get(i) {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.sections[0].value.clone_from(text_line);
            }
        }
    }
}

pub fn debug_overlay_visibility_system(
    root_query: Query<Entity, With<DebugOverlayUI>>,
    mut visibility: Query<&mut Visibility>,
    visible: Res<DebugOverlayVisible>,
) {
    if let Ok(root) = root_query.get_single() {
        if let Ok(mut vis) = visibility.get_mut(root) {
            *vis = if visible.0 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

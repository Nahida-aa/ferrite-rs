## Goal
Provide a working Minecraft client with world selection, server launch, LAN discovery, and connection flow.

## Constraints & Preferences
- `online_mode = false` in dev builds, `true` in release (`!cfg!(debug_assertions)`)
- Ferrumc output prefixed with `[ferrumc]` for terminal visibility
- World data scanned from `saves/` (Anvil format), imported to LMDB on first use
- Log files per-run in `logs/` directory, timestamped
- Mouse cursor only grabbed when `net.connected`, not on menus
- UI font: `aaxlMonoSC-Regular.ttf` (user's chosen CJK-supporting font)

## Progress
### Done
- Root cause found: `write_server_config` wrote to `configs/config.toml` (project root), but ferrumc's `get_root_path()` = `current_exe().parent()` resolves to `<binary_parent>/`. Config was never read by ferrumc → used embedded defaults (`online_mode = true`) → authentication errors.
- Binary copy to root: `ensure_local_binary()` copies `ferrumc/target/release/ferrumc` → `./ferrite-server`. Now `get_root_path()` = `.` (project root), config at `./configs/config.toml` matches.
- `write_server_config()` now takes `root: &Path` parameter (provided by `ServerHandle::spawn()`)
- World name (`db_path`) threaded through `PendingConnect` → `connect_to_server` → `ServerHandle::spawn()`
- Anvil import support: `WorldManager::needs_import()` checks for `region/` dir without `data.mdb`; `ServerHandle::spawn()` runs `ferrumc import --import-path=...` as subprocess first if needed
- World scan from `saves/` (vanilla Minecraft save directory)
- Per-run log files: `logs/ferrite-YYYYMMDD-HHMMSS.log`
- Exclusive system `handle_connections` removed — split into `handle_pending_connect` + `poll_server_startup` (regular systems). `ServerHandle::spawn()` runs on background thread, frame loop no longer blocked
- `_join` handle from `Network::connect()` now saved in `NetworkRes.net_join`, aborted on disconnect (button or remote)
- `ChunkEntities` resource (`HashMap<(i32,i32), Entity>`) tracks spawned chunk meshes — deduplicates on re-receive, despawns all on disconnect
- All Update systems chained via `.chain()`
- "Multi Player" button navigates to `ServerList` screen
- `killin` uses `pkill -x ferrumc` + `pkill -x ferrite-server` (exact process name match)
- `PauseMenuOpen` reset on remote disconnect in `handle_network_events_system`
- Unused Cargo dependencies removed
- Dead code removed: `InfoText` component, duplicated `btn()`, unused imports
- `Color::rgb()` → `Color::srgb()` in all UI code
- Font restored to `aaxlMonoSC-Regular.ttf` (user's original CJK font). `UiFont` resource loaded via Bevy `AssetServer` at startup
- `server-cli` crate created with `scan-lan` subcommand (CLI-only LAN scan, no Bevy dependency)
- LAN discovery format corrected: vanilla Minecraft 1.21.8 uses `[MOTD]...[/MOTD][AD]...[/AD]` XML-style tags, not null-separated fields. Packets previously mistaken for "Bedrock" were actually Java Edition broadcasts
- Shared LAN types (`DiscoveredServer`, `parse_lan_packet`, `create_lan_socket`) extracted to `ferrite-net/src/lan.rs`. Both `crates/client/` and `crates/server-cli/` import from `ferrite_net::lan`
- World selection UI changed from direct-connect to click-to-select + "Play World" button. Selected world highlighted (yellow text), play button turns green when enabled
- UI rebuild uses generation counter: only rebuilds when new servers discovered, not every frame (eliminates flicker)
- `ferrite-gui` crate extracted from `server-client`: `player.rs`, `worlds.rs`, `lan_discovery.rs`, `ui/mod.rs` + `ui/menu.rs`, `ui/hud.rs`, `ui/pause.rs`, `ui/server_list.rs` moved to new crate.
- Shared types (`LanDiscoveryState`, `CmdTx`, `PlayerPlugin`, `UIPlugin`) defined in `ferrite-gui`; client imports them with `ferrite_gui::*`.
- Old files removed from client: `ui.rs`, `ui/`, `worlds.rs`, `player.rs`, `lan_discovery.rs`, `game.rs`.

### In Progress
- (none)

### Blocked
- (none)

## Key Decisions
- Binary location: copy to `./ferrite-server` instead of `./ferrumc` to avoid conflict with submodule directory
- World data path: `saves/<name>/` for both Anvil (original) and LMDB (imported), co-located
- Import on demand: only runs if `region/` exists but `data.mdb` doesn't; after first import, subsequent starts skip it
- Log rotation: per-run file in `logs/` folder, timestamped
- Server startup on background thread to avoid blocking Bevy frame loop, polled via `PendingServerSpawn` resource
- LAN discovery: always-on UDP socket (init once at first use, never close) to avoid bind/release race
- LAN sharing: `create_lan_socket` in `ferrite-net::lan` shared by both CLI and client
- World select: click-to-select (not direct-connect), "Play World" button activates on selection, generation-based rebuild (MC-style)
- GUI extraction: `ferrite-gui` as separate crate with all UI + player + worlds + lan types; client keeps only `net_plugin`, `server`, `chunk_mesh`, `main`

## Next Steps
1. Add "Direct Connect" input field in ServerList UI for manual IP:port entry
2. Handle `online_mode=true` remote servers (Mojang/Microsoft authentication)
3. Polish chunk mesh rendering (chunk loading/unloading based on player position)

## Critical Context
- `get_root_path()` = `current_exe().parent()` — binary must be at project root for config paths to align with CWD
- `std::fs::copy` fails with "Is a directory" when target `./ferrumc` exists as directory (submodule) — must use non-conflicting name
- Import subprocess pipes `[ferrumc]`-prefixed output to terminal, then waits for exit before starting server
- Ferrumc still needs nightly to rebuild; current binary at `ferrumc/target/release/ferrumc` works without recompilation
- `Network::connect()` requires `&tokio::runtime::Handle` (take from `EcsRuntime` resource)
- LAN discovery uses Minecraft Java Edition protocol: multicast group `224.0.2.60` port `4445`, format `[MOTD]<motd>[/MOTD][AD]<port>[/AD]` as UTF-8 bytes

## Relevant Files
- `crates/client/src/main.rs`: entry point, `--auto-connect` flag, log init
- `crates/client/src/net_plugin.rs`: `NetworkRes`, `PendingServerSpawn`, `ChunkEntities`; all Update systems defined and chained; imports `ferrite_gui::*`
- `crates/client/src/server.rs`: `ServerHandle::spawn(db_path)`, `find_ferrumc()`, `ensure_local_binary()`, `killin()`
- `crates/client/src/chunk_mesh.rs`: greedy meshing for chunk rendering
- `crates/ferrite-gui/src/lib.rs`: all shared UI/player/world/lan type definitions; `pub mod` declarations
- `crates/ferrite-gui/src/player.rs`: `PlayerPlugin`, `CmdTx`, player movement/look systems
- `crates/ferrite-gui/src/worlds.rs`: `WorldManager`, `SelectedWorld`, `write_server_config(root, db_path)`
- `crates/ferrite-gui/src/lan_discovery.rs`: `LanState` (background UDP listener)
- `crates/ferrite-gui/src/ui/mod.rs`: `UIPlugin` definition; inserts UI resources
- `crates/ferrite-gui/src/ui/menu.rs`: `spawn_menu()`, `spawn_world_select()`, highlight/button update systems
- `crates/ferrite-gui/src/ui/server_list.rs`: `spawn_server_list()`, `update_server_list()` (generation-based), `LanServerButton`
- `crates/ferrite-gui/src/ui/hud.rs`: HUD with entity ID / game mode display
- `crates/ferrite-gui/src/ui/pause.rs`: pause menu (Back to Game, Disconnect)
- `crates/ferrite-net/src/lan.rs`: shared `DiscoveredServer`, `parse_lan_packet`, `create_lan_socket`
- `crates/server-cli/src/scan_lan.rs`: CLI-only LAN scan using `ferrite_net::lan`
- `crates/server-cli/src/main.rs`: `scan-lan` subcommand with `clap`

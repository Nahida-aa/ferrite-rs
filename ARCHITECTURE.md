# 当前架构与计划

## 现状

- `ferrite-core`、`network`、`client` 已拆分清楚。
- 客户端已经进入 Bevy ECS，网络事件也已经走 Bevy Event。
- UI 当前使用 `bevy_ui`，不是 `bevy_egui`。
- 协议编解码保留在 `ferrite-core`，传输层留在 `network`。

## 已确认的方向

1. **主 UI 用 `bevy_ui`** — 菜单、HUD、暂停界面都继续走 Bevy 原生 UI。
2. **核心逻辑先稳定** — 继续把网络、玩家、世界状态整理成 ECS 资源和系统。
3. **每步都可运行** — 任何重构都尽量保持能编译、能启动、能联机。

## 当前进度

- `ferrite-core`
  - 共享协议包、VarInt、字符串、UUID 编解码。
  - 只放纯逻辑，不依赖 Bevy。
- `network`
  - TCP、加密、压缩、分包、登录/配置/游戏状态机。
- `client`
  - Bevy App、玩家状态、网络事件、`bevy_ui` 界面。

## 关键文件

- [crates/client/src/main.rs](crates/client/src/main.rs) — 客户端入口。
- [crates/client/src/game.rs](crates/client/src/game.rs) — 游戏插件组装。
- [crates/client/src/net_plugin.rs](crates/client/src/net_plugin.rs) — 网络事件与 ECS 同步。
- [crates/client/src/ui.rs](crates/client/src/ui.rs) — 菜单、HUD、暂停界面。
- [crates/network/src/network/connection.rs](crates/network/src/network/connection.rs) — 网络连接与状态机。
- [crates/ferrite-core/src/protocol/codec.rs](crates/ferrite-core/src/protocol/codec.rs) — 协议编解码复用点。

## 接下来推进

### 1. UI 继续收敛

- 菜单、HUD、暂停菜单统一整理成更清晰的 Bevy UI 结构。
- 交互逻辑尽量用组件 + 系统，不再回到单体状态机。

### 2. 世界与渲染

- 把区块、实体、相机状态继续 ECS 化。
- 逐步把游戏画面和数据流拆清，减少手写状态耦合。

### 3. 游戏玩法

- 玩家移动、断线重连、服务器同步、基础 HUD 继续补齐。
- 先做稳定可玩，再考虑更复杂的视觉效果。

## 结论

现在这条线更适合走：**`bevy_ui` 作为主 UI，Bevy ECS 作为主架构**
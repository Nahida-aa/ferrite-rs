# Bevy 迁移路线图

## 现状

- `ferrite-core` + `ferrite-client` 共 ~2200 行 Rust
- 已依赖 `bevy_ecs 0.14`，但零使用
- 架构：`AppState` 单体调度器，winit 事件循环驱动
- 渲染：手写 wgpu 管线 + egui 覆盖层
- 网络：tokio 独立任务 + mpsc 通道

## 设计原则

1. **每步代码可运行** — 不允许出现两周无法启动的 PR
2. **ferrite-core 不动** — 协议层、Codec、世界数据模型是纯逻辑，与 Bevy 无关
3. **egui 保留或等价替代** — 不因迁移 UI 而阻塞主逻辑

---

## Phase 0：架构解耦（1–2 天）

**目标**：在不改渲染框架的前提下，把单体 AppState 拆成 ECS 友好的模块。

| 任务 | 改动 | 代码可用性 |
|---|---|---|
| 0.1 NetworkEvent 迁移 | 把 NetworkEvent 枚举从 `mpsc` 改为 `bevy_ecs::Event`，主线程每帧 drain 到 Bevy EventWriter | 正常 |
| 0.2 AppState → Resource | `AppState` 拆成 `NetworkState`/`PlayerState`/`ServerState` 等 `Resource` | 正常 |
| 0.3 Action queue → Event | `Action` 枚举改成 `bevy_ecs::Event`，处理逻辑变成 System | 正常 |
| 0.4 Renderer 独立 | `state.rs` 不再直接管理 Renderer，改由 Bevy 插件持有 | 正常 |

**出口条件**：`bevy_ecs::World` 创建，AppState 的数据全部在 Resource 中，
winit 循环仍然驱动渲染，但逻辑已是 ECS System。

---

## Phase 1：Bevy 内核替换（2–3 天）

**目标**：winit 事件循环 → `Bevy::new().run()`。

| 任务 | 改动 | 风险 |
|---|---|---|
| 1.1 替换入口 | `main.rs`: 删除 winit loop，改为 `App::new().add_plugins(DefaultPlugins).run()` | 窗口不显示/崩溃 |
| 1.2 禁用 DefaultPlugins 的部分 | 自定义渲染循环（保留手写 wgpu），用 `RenderApp` 或自定义 `RenderPlugin` | 需要理解 Bevy RenderGraph |
| 1.3 egui 集成 | 用 `bevy_egui` 或手写 Bevy 插件包装现有 egui 状态 | bevy_egui 版本与 Bevy 绑定 |
| 1.4 窗口配置 | `WindowPlugin` 配置窗口标题、大小 | 简单 |

**关键决策点**：Bevy 渲染 vs 手写渲染

**方案 A (推荐)**：Bevy 渲染 + 3D PBR
- 优势：免费获得相机、光照、材质、场景管理
- 代价：需要把 wgpu 手写管线换成 Bevy Material/ Mesh
- 工作估：中等

**方案 B**：手写 wgpu 渲染 + Bevy ECS/App 外壳
- 优势：现有渲染代码直接搬
- 代价：Bevy 最核心的价值（渲染栈）完全没用上，白迁移

**出口条件**：Bevy 启动，窗口弹出，egui 菜单正常工作。

---

## Phase 2：3D 渲染迁移（2–4 天）

**目标**：让方块通过 Bevy 渲染而非手写 wgpu。

| 任务 | 改动 | 依赖 |
|---|---|---|
| 2.1 Camera | 删除手写 `CameraUniform` + `nalgebra`，改用 `Camera3dBundle` + `Transform` | Phase 1 |
| 2.2 Mesh + Material | 草方块从 `Vertex`/`INDICES` 改为 `Mesh::from()` + 自定义 `Material` | Phase 1 |
| 2.3 光照 | Bevy `PointLight` / `DirectionalLight` 代替硬编码颜色 | Phase 2.2 |
| 2.4 深度纹理 | Bevy 自动管理 depth attachment，删除手写 `depth_texture` | Phase 2.2 |

**可跳过选项**：直接把手写 wgpu 封装成 Bevy `ExternalRenderer`，3D 部分后续再慢慢拆。
这可以缩短 Phase 2 为 1 天。

**出口条件**：窗口中有一个通过 Bevy 渲染的方块，光照正常。

---

## Phase 3：网络集成（1–2 天）

**目标**：网络事件成为 Bevy 一等公民。

| 任务 | 改动 | 依赖 |
|---|---|---|
| 3.1 网络 Plugin | `NetworkPlugin` 管理 tokio runtime + mpsc | Phase 0 |
| 3.2 ServerHandle Plugin | `ServerPlugin` 管理 FerrumC 子进程 | Phase 0 |
| 3.3 网络 System | 每帧 `EventReader<NetworkEvent>` 处理协议数据 | Phase 0 |
| 3.4 状态迁移 | `NetworkEvent::Connected/Disconnected` → Bevy `State` 切换 | Phase 3.1 |

**出口条件**：网络连接 + 断开触发 Bevy 状态切换，Play 状态下服务器数据流入 Bevy ECS。

---

## Phase 4：功能重建（3–7 天）

**目标**：恢复并超越现有的全部功能。

| 任务 | 改动 | 依赖 |
|---|---|---|
| 4.1 HUD | egui 覆盖 Bevy 渲染（`bevy_egui`），保留坐标/准星/血量 | Phase 1 |
| 4.2 WASD 移动 | `Input<KeyCode>` + `Transform` → C2S 位置包 | Phase 3 |
| 4.3 区块渲染 | 解析 Chunk Data (0x27)，写入 Bevy Mesh/InstanceBuffer | Phase 2 |
| 4.4 音频 | `kira` → `bevy_kira_audio` | Phase 0 |
| 4.5 玩家实体 | `PlayerBundle`（Position, Health, Gamemode）| Phase 3 |

**出口条件**：全功能 Parity。

---

## 总时间估算

| Phase | 天数 | 可行性 |
|---|---|---|
| 0 架构解耦 | 1–2 | 安全，代码每次可编译运行 |
| 1 Bevy 内核 | 2–3 | 有断档风险（egui 集成容易卡住） |
| 2 渲染迁移 | 2–4 | 可选简化路径可大幅降风险 |
| 3 网络集成 | 1–2 | 低风险，Phase 0 走完就剩 Plugin 封装 |
| 4 功能重建 | 3–7 | 跟 Phase 2 深度绑定 |
| **总计** | **9–18 天** | |

## 风险

1. **egui 与 Bevy 版本兼容**
   - `bevy_egui` 的版本锁定：Bevy 0.14 → bevy_egui 0.31
   - 如果 bevy_egui 不兼容，需要自己写 egui-wgpu 集成到 Bevy RenderGraph
   - **缓解**：Phase 0 保留 egui 独立，Phase 1 才集成，留出解决空间

2. **Bevy wgpu 版本冲突**
   - 我们手写用 wgpu 0.20，Bevy 0.14 依赖的是 wgpu 23+
   - Phase 1 之前需要升级 wgpu → 0.23+，否则无法共存
   - **缓解**：Phase 0 同时升级 wgpu。或 Phase 1 两套 wgpu 共存（不推荐但可行）

3. **两个 wgpu 实例**
   - Bevy 有自己的设备/队列，我们手写也有
   - **推荐**：一旦 Phase 1 完成，手写 wgpu 代码就逐渐被替代，不长期维护两套

## 建议步骤

```
现在
  │
  ├─ 0.1 NetworkEvent → bevy Event  （不改渲染，立即测试）
  ├─ 0.2 wgpu 升级 0.20 → 0.23+   （与 Bevy 对齐）
  ├─ 0.3 AppState 拆 Resource       （ECS 化）
  │
  ├─ 1.1 升级到 Bevy 0.14 DefaultPlugins
  ├─ 1.2 egui 复活
  │
  ├─ 2.1 手写方块 → Bevy Mesh/Material
  │
  └─ 并行推进 Phase 3 + 4
```

---


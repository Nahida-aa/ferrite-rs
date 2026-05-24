# Ferrite

Rust 编写的 Minecraft 客户端。

## 当前架构

仓库按「共享协议层」与「客户端运行时」拆分：

- 详细说明见 [ARCHITECTURE.md](ARCHITECTURE.md)。

```mermaid
flowchart LR
	MC[兼容 Minecraft 协议的服务端] --> NET[ferrite-net\n TCP + 加密 + 压缩 + 分包]
	NET --> CORE[ferrite-core\n 协议包, 编解码, 通用类型]
	CORE --> ECS[Bevy ECS\n 玩家状态 / UI / 游戏逻辑]
	ECS --> RENDER[渲染与界面\n wgpu + bevy + bevy_ui + kira]
```

### 工作区划分

- [crates/ferrite-core](crates/ferrite-core) 负责共享的协议模型和基础工具：NBT、Block、Chunk、Position、协议包定义、VarInt/字符串/UUID 编解码。
- [crates/ferrite-net](crates/ferrite-net) 负责网络连接、加密、压缩、帧处理和登录/配置/游戏状态机。
- [crates/client](crates/client) 负责客户端运行时：Bevy App、网络连接、ECS 状态、UI、玩家实体、渲染与输入。
- [crates/cli](crates/cli) 负责命令行工具。

### 运行链路

1. [crates/client/src/main.rs](crates/client/src/main.rs) 初始化日志、Bevy App，并可选执行自动连接。
2. [crates/client/src/game.rs](crates/client/src/game.rs) 组装游戏插件，挂载网络、玩家、UI 模块。
3. [crates/client/src/net_plugin.rs](crates/client/src/net_plugin.rs) 管理网络任务、事件轮询和 ECS 状态同步。
### 设计原则


- 通用协议字节编解码放在 ferrite-core，避免 client 重复实现。
- 加密、压缩、读取网络帧等传输层逻辑放在 ferrite-net。
- 业务状态通过 Bevy ECS 和消息通道在网络层与渲染/UI 层之间流动。

## 技术栈

| 领域 | 选型 |
|------|------|
| ECS | bevy_ecs / Bevy |
| 渲染 | wgpu |
| UI | bevy_ui |
| 音频 | kira |
| 网络 | tokio |
| 序列化 | serde |
| 日志 | tracing |
| 错误处理 | anyhow + thiserror |

## 运行

```bash
# 发行版启动客户端
npm run start
# or
bun run start
```

其他命令请查看 [package.json](package.json)。

## 依赖服务端

客户端需要一个兼容 Minecraft 协议的服务端。当前开发常用 [FerrumC](https://github.com/sweattypalms/ferrumc)，也可以接入其他兼容实现。

1.21.8
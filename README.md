# Ferrite

Rust 编写的 Minecraft 客户端。

## 仓库结构

```
ferrite-rs/
├── lib/
│   └── ferrite-core/    共享类型: NBT, 协议包, Block, Chunk, 编解码
├── client/               客户端 (wgpu + egui + bevy_ecs)
└── cli/                  命令行工具
```

服务端使用 [FerrumC](https://github.com/sweattypalms/ferrumc) 或其他兼容 MC 协议的服务端。

## 技术栈

| 领域 | 选型 |
|------|------|
| ECS | bevy_ecs |
| 渲染 | wgpu |
| UI | egui |
| 音频 | kira |
| 网络 | tokio |
| 序列化 | serde |
| 日志 | tracing |
| 错误处理 | anyhow + thiserror |

## 构建

```bash
cargo build --release
```

## 运行

```bash
# 客户端
cargo run --bin ferrite-client

# CLI
cargo run --bin ferrite-cli
```

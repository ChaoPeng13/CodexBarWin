# CodexBar Windows

> Windows 版 AI Code Plan Token 用量追踪器 — 系统托盘常驻，一目了然

灵感来源于 macOS [CodexBar](https://github.com/steipete/codexbar)，基于 **Tauri 2 + React + Rust** 构建，针对国内 AI 厂商提供的 Code Plan 会员，提供用量追踪。目前已支持智谱 GLM Coding Plan 会员和 Kimi 会员。

## ✨ 功能

- **系统托盘常驻** — 左键单击弹出用量面板，右键显示菜单
- **多 Provider 支持** — 智谱 GLM、Kimi (月之暗面)
- **实时用量展示** — Token 用量、配额、带进度条
- **会员权益展示** — 显示当前套餐、订阅状态、功能权益列表
- **定时自动刷新** — 可配置刷新间隔（30s ~ 10min）
- **开机自启动** — 可选 Windows 登录时启动
- **深色界面** — 现代深色 UI，不干扰工作流

## 🚀 快速开始

### 环境要求

- [Node.js 18+](https://nodejs.org/)
- [Rust 1.70+](https://rustup.rs/)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（Windows 11 已内置）

### 开发模式

```powershell
# 安装依赖
npm install

# 启动开发服务器（热重载）
npm run tauri dev
```

### 构建发布版

```powershell
npm run tauri build
# 输出位于 src-tauri/target/release/bundle/
```

## ⚙️ Provider 配置

每个 Provider 需要单独配置认证方式：

| Provider                  | 认证方式       | 获取方法                                                                                                 |
| ------------------------- | -------------- | -------------------------------------------------------------------------------------------------------- |
| **智谱 GLM**        | API Key        | [bigmodel.cn](https://bigmodel.cn) → 用户中心 → 查看 API Key                                              |
| **Kimi (月之暗面)** | Session Token  | [kimi.com/code/console](https://kimi.com/code/console) → F12 → Network → 复制 Authorization Bearer Token |
| Minimax                   | 待充会员后实现 |                                                                                                          |
| 其它会员制的大模型提供商  |                |                                                                                                          |

### 认证方式说明

#### API Key（智谱 GLM）

1. 登录对应平台官网
2. 进入 API Keys 管理页面
3. 创建或复制现有 API Key
4. 粘贴到 CodexBar 设置中

#### Session Token（Kimi 月之暗面）

Kimi 使用 Session Token 获取详细的用量统计（本周用量、频限、会员权益等）：

1. 打开 [kimi.com/code/console](https://kimi.com/code/console) 并登录
2. 按 F12 打开 DevTools → Network 标签
3. 刷新页面，找到 `GetSubscription` 或 `GetUsages` 请求
4. 复制 Request Headers 中的 `Authorization` 字段值（格式为 `Bearer eyJ...`）
5. 粘贴到 CodexBar 设置中的 **Session Token** 字段

> **注意**：Kimi 也支持配置 Moonshot 开放平台 API Key（`platform.moonshot.cn`），但只能查询余额，无法获取详细用量统计。建议优先使用 Session Token。

## 📁 项目结构

```
CodexBarWin/
├── src/                    # React 前端
│   ├── components/
│   │   ├── ProviderCard.tsx    # 用量展示卡片
│   │   ├── ZhipuCard.tsx       # 智谱 GLM 用量卡片
│   │   ├── KimiCard.tsx        # Kimi 用量卡片
│   │   └── SettingsPanel.tsx   # 设置面板
│   ├── hooks/
│   │   └── useUsageData.ts     # 数据获取 Hook
│   ├── App.tsx                 # 主界面
│   └── types.ts                # TypeScript 类型
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── providers/          # 各 Provider 拉取逻辑
│       │   ├── zhipu.rs        # 智谱 GLM
│       │   └── kimicode.rs     # Kimi (月之暗面)
│       ├── commands.rs         # Tauri 命令（前后端桥接）
│       ├── config.rs           # 配置持久化
│       ├── tray.rs             # 系统托盘
│       ├── models.rs           # 数据结构定义
│       └── main.rs             # 入口
└── package.json
```

## 🛠️ 添加新 Provider

1. 在 `src-tauri/src/providers/` 新建 `myprovider.rs`
2. 实现 `async fn fetch(config: &ProviderConfig) -> UsageData`
3. 在 `providers/mod.rs` 添加 `pub mod myprovider;`
4. 在 `commands.rs` 的 `fetch_all_usage` 中添加并行调用
5. 在 `src/components/SettingsPanel.tsx` 的 `PROVIDERS` 数组中添加配置项

## 📝 配置文件

配置保存在 `%APPDATA%\CodexBar\config.json`，Cookie 等敏感信息直接存储，不涉及系统 Keychain。

## 🚀 开发工具

本项目采用 **[WorkBuddy](https://www.codebuddy.cn)** 进行开发。

WorkBuddy 是一款 AI 驱动的智能开发助手，为开发者提供：

- **智能代码生成** — 根据需求自动生成高质量代码
- **架构设计建议** — 提供技术选型和架构优化方案
- **代码审查与重构** — 发现潜在问题，提升代码质量
- **跨语言支持** — 支持 Rust、TypeScript、React 等全栈技术栈
- **上下文感知** — 理解项目结构，提供精准的代码建议

### 使用 WorkBuddy 开发本项目的主要场景

- **Provider 接入** — 快速实现新的 AI 平台用量查询接口
- **UI 组件开发** — 生成和优化 React 组件及样式
- **类型定义同步** — 保持 Rust 后端与 TypeScript 前端类型一致
- **文档维护** — 自动生成和更新项目文档

## 📄 License

MIT

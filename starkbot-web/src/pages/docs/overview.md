---
name: StarkBot Documentation
---

StarkBot is a self-hosted AI agent platform that connects to messaging platforms, executes tools, remembers context, and integrates with Web3.

## What is StarkBot?

A Rust-powered backend with a React dashboard that turns AI models into autonomous agents:

- **Multi-platform** — Telegram, Slack, Discord, and web chat
- **Tool execution** — Web search, file ops, shell commands, blockchain transactions
- **Persistent memory** — Cross-session context with automatic consolidation
- **Scheduled tasks** — Cron jobs and heartbeat automation
- **Web3 native** — Wallet auth, x402 payments, on-chain identity

## Core Capabilities

| Feature | Description |
|---------|-------------|
| **Channels** | Connect multiple platforms simultaneously |
| **AI Providers** | Claude, OpenAI, Llama with streaming and tool calling |
| **40+ Tools** | Web, filesystem, exec, messaging, and blockchain |
| **Skills** | Extend capabilities with custom markdown modules |
| **Memory** | Facts, preferences, tasks, and daily logs |
| **Real-time** | WebSocket events for tool progress and transactions |

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────┐
│            Telegram · Slack · Discord · Web          │
└─────────────────────────┬───────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────┐
│              Message Dispatcher (Rust)              │
│  normalize → context → AI → tools → memory → reply  │
└─────────────────────────┬───────────────────────────┘
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
     ┌─────────┐    ┌──────────┐    ┌──────────┐
     │   AI    │    │  Tools   │    │  SQLite  │
     │ Claude  │    │ Registry │    │ Database │
     │ OpenAI  │    │  40+     │    │          │
     └─────────┘    └──────────┘    └──────────┘
```

## Quick Links

| Section | What You'll Learn |
|---------|-------------------|
| [Getting Started](/docs/getting-started) | Run StarkBot in 5 minutes |
| [Architecture](/docs/architecture) | System design deep dive |
| [Tools](/docs/tools) | Built-in capabilities |
| [Skills](/docs/skills) | Custom extensions |
| [Channels](/docs/channels) | Platform integrations |
| [Scheduling](/docs/scheduling) | Automation and cron |
| [Memories](/docs/memories) | Long-term context |
| [Configuration](/docs/configuration) | Environment and settings |
| [API Reference](/docs/api) | REST and WebSocket APIs |

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust · Actix-web · Tokio |
| Frontend | React · TypeScript · Vite |
| Database | SQLite |
| WebSocket | tokio-tungstenite |
| Styling | Tailwind CSS |
| AI | Anthropic Claude · OpenAI · Llama |
| Auth | Sign In With Ethereum (SIWE) |

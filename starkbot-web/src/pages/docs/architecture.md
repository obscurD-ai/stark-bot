---
name: Architecture
---

StarkBot is a modular system with a Rust backend, React frontend, and real-time WebSocket communication.

## System Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    External Platforms                         │
│       Telegram        Slack        Discord        Web         │
└───────────┬─────────────┬────────────┬─────────────┬─────────┘
            │             │            │             │
            ▼             ▼            ▼             ▼
┌──────────────────────────────────────────────────────────────┐
│                     Channel Handlers                          │
│     telegram.rs    slack.rs    discord.rs    REST API         │
└───────────────────────────┬──────────────────────────────────┘
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                   Message Dispatcher                          │
│                                                               │
│   1. Normalize message    4. Execute tool loop (max 10)       │
│   2. Load identity        5. Extract memory markers           │
│   3. Build AI context     6. Store history & respond          │
└───────────────────────────┬──────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         ▼                  ▼                  ▼
    ┌─────────┐       ┌──────────┐       ┌──────────┐
    │   AI    │       │  Tools   │       │  SQLite  │
    │ Client  │       │ Registry │       │ Database │
    │         │       │          │       │          │
    │ Claude  │       │ Web      │       │ Sessions │
    │ OpenAI  │       │ Files    │       │ Memories │
    │ Llama   │       │ Exec     │       │ Channels │
    │         │       │ Web3     │       │ Skills   │
    └─────────┘       └──────────┘       └──────────┘
```

---

## Backend (Rust + Actix-web)

### Entry Point

The backend initializes services in order:

1. Load environment configuration
2. Initialize SQLite database with migrations
3. Create tool registry (40+ built-in tools)
4. Create skill registry (custom extensions)
5. Start WebSocket gateway (port 8081)
6. Start HTTP server (port 8080)
7. Auto-start enabled channels

### Message Dispatcher

The core message processing engine:

```
Message → Normalize → Identity → Context → AI → Tools → Memory → Response
```

| Step | Action |
|------|--------|
| **Normalize** | Convert platform message to standard format |
| **Identity** | Get or create user identity across platforms |
| **Context** | Load session history + relevant memories |
| **AI** | Send to configured provider with tool definitions |
| **Tools** | Execute tool calls in loop (up to 10 iterations) |
| **Memory** | Extract `[REMEMBER:]` markers and store |
| **Response** | Send back to originating platform |

### AI Providers

Unified interface supporting multiple providers:

| Provider | Features |
|----------|----------|
| **Claude** | Tool calling, extended thinking, streaming |
| **OpenAI** | Tool calling, streaming, x402 payment support |
| **Llama** | Local/Ollama, custom endpoints |

### Tool Registry

Tools organized by group and access level:

| Group | Tools |
|-------|-------|
| **Web** | `web_search`, `web_fetch` |
| **Filesystem** | `read_file`, `write_file`, `list_files`, `glob`, `grep` |
| **Exec** | `exec`, `git` |
| **Messaging** | `agent_send`, `say_to_user` |
| **Web3** | `web3_tx`, `token_lookup`, `x402_fetch` |
| **System** | `subagent`, `memory_store`, `modify_soul` |

### WebSocket Gateway

Real-time event broadcasting (port 8081):

| Event | Description |
|-------|-------------|
| `agent.tool_call` | Tool execution started |
| `tool.result` | Tool completed with result |
| `agent.thinking` | AI processing indicator |
| `tx.pending` | Blockchain transaction pending |
| `tx.confirmed` | Transaction confirmed |
| `confirmation.required` | User approval needed |

---

## Frontend (React + TypeScript)

### Tech Stack

- **React 18** with TypeScript
- **Vite** for builds and hot-reload
- **Tailwind CSS** for styling
- **React Router** for navigation
- **Ethers.js** for wallet integration

### Page Structure

| Page | Purpose |
|------|---------|
| **Dashboard** | Stats and quick actions |
| **Agent Chat** | Real-time conversation interface |
| **Channels** | Platform connections |
| **Agent Settings** | AI model configuration |
| **Tools** | Browse available tools |
| **Skills** | Upload and manage skills |
| **Scheduling** | Cron jobs and automation |
| **Sessions** | Conversation history |
| **Memories** | Long-term storage browser |

### Real-Time Updates

The frontend maintains a WebSocket connection:

```typescript
useGateway({
  onToolCall: (e) => showProgress(e),
  onToolResult: (e) => updateResults(e),
  onConfirmation: (e) => promptUser(e)
});
```

---

## Data Flow

### Chat Message

```
1. User types "Search for Rust news"
        ↓
2. POST /api/chat with message + session
        ↓
3. Dispatcher normalizes and builds context
        ↓
4. AI decides to call web_search tool
        ↓
5. WebSocket broadcasts agent.tool_call
        ↓
6. Tool executes, returns results
        ↓
7. WebSocket broadcasts tool.result
        ↓
8. AI generates final response
        ↓
9. Response stored in session
        ↓
10. Response sent to user
```

### Scheduled Job

```
1. Scheduler checks every 10 seconds
        ↓
2. Job with cron "0 9 * * MON" is due
        ↓
3. Dispatcher processes like regular message
        ↓
4. Response logged to job history
        ↓
5. Next run time calculated
```

---

## Security Model

| Layer | Mechanism |
|-------|-----------|
| **Authentication** | SIWE (Sign In With Ethereum) |
| **Authorization** | Single admin wallet address |
| **API Keys** | Encrypted at rest in SQLite |
| **Tool Safety** | Dangerous commands blocklisted |
| **Session Tokens** | JWT with expiration |

---

## Database Schema

Key tables in SQLite:

| Table | Purpose |
|-------|---------|
| `auth_sessions` | JWT tokens and expiration |
| `identity_links` | Cross-platform user mapping |
| `chat_sessions` | Conversation contexts |
| `session_messages` | Message history |
| `memories` | Long-term facts and preferences |
| `external_channels` | Platform configurations |
| `skills` | Custom skill definitions |
| `cron_jobs` | Scheduled tasks |
| `agent_settings` | AI model configuration |

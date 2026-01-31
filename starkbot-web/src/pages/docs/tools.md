---
name: Tools
---

Tools let the AI agent take actions beyond conversation. StarkBot includes 40+ built-in tools across six categories.

## How Tools Work

```
User Message → AI Analyzes → Tool Call → Execute → Result → AI Continues
```

The AI can chain up to 10 tool calls per message to complete complex tasks.

---

## Web Tools

### web_search

Search the web using Brave Search or SerpAPI.

```json
{
  "name": "web_search",
  "parameters": {
    "query": "Rust async runtime comparison 2024"
  }
}
```

Requires: Brave Search or SerpAPI key in API Keys.

### web_fetch

Fetch and parse content from a URL.

```json
{
  "name": "web_fetch",
  "parameters": {
    "url": "https://example.com/api/data",
    "selector": ".main-content"
  }
}
```

---

## Filesystem Tools

### read_file

Read file contents.

```json
{ "name": "read_file", "parameters": { "path": "./src/main.rs" } }
```

### write_file

Create or overwrite a file.

```json
{
  "name": "write_file",
  "parameters": {
    "path": "./output.txt",
    "content": "Hello, World!"
  }
}
```

### list_files

List directory contents.

```json
{ "name": "list_files", "parameters": { "path": "./src", "recursive": true } }
```

### glob

Find files matching a pattern.

```json
{ "name": "glob", "parameters": { "pattern": "**/*.rs" } }
```

### grep

Search file contents.

```json
{ "name": "grep", "parameters": { "pattern": "TODO", "path": "./src" } }
```

### apply_patch

Apply a unified diff patch to a file.

```json
{
  "name": "apply_patch",
  "parameters": {
    "path": "./src/lib.rs",
    "patch": "@@ -1,3 +1,4 @@\n+// New comment\n fn main() {"
  }
}
```

---

## Exec Tools

### exec

Execute shell commands.

```json
{
  "name": "exec",
  "parameters": {
    "command": "cargo build --release",
    "cwd": "./project",
    "timeout": 60000
  }
}
```

**Safety:** Dangerous commands are blocked. Shell metacharacters are restricted.

### git

Git operations with built-in safety.

```json
{ "name": "git", "parameters": { "command": "status" } }
{ "name": "git", "parameters": { "command": "diff HEAD~1" } }
```

---

## Messaging Tools

### agent_send

Send a message to any configured channel.

```json
{
  "name": "agent_send",
  "parameters": {
    "channel_id": "discord-channel-uuid",
    "message": "Build completed successfully!"
  }
}
```

### say_to_user

Reply in the current conversation.

```json
{ "name": "say_to_user", "parameters": { "message": "Working on it..." } }
```

### ask_user

Request user confirmation before proceeding.

```json
{
  "name": "ask_user",
  "parameters": {
    "question": "Deploy to production?",
    "options": ["Yes", "No"]
  }
}
```

---

## Web3 Tools

### web3_tx

Send a blockchain transaction.

```json
{
  "name": "web3_tx",
  "parameters": {
    "to": "0x1234...",
    "value": "0.1",
    "data": "0x..."
  }
}
```

### token_lookup

Get token information.

```json
{ "name": "token_lookup", "parameters": { "address": "0x1234...", "chain": "base" } }
```

### x402_fetch

Fetch from a pay-per-use API with automatic USDC payment.

```json
{
  "name": "x402_fetch",
  "parameters": {
    "url": "https://api.example.com/premium",
    "method": "GET"
  }
}
```

---

## System Tools

### subagent

Spawn a background agent for parallel tasks.

```json
{
  "name": "subagent",
  "parameters": {
    "task": "Research competitor pricing",
    "tools": ["web_search", "web_fetch"]
  }
}
```

### memory_store

Explicitly store a memory.

```json
{
  "name": "memory_store",
  "parameters": {
    "content": "User prefers TypeScript over JavaScript",
    "memory_type": "preference"
  }
}
```

### modify_soul

Update the agent's personality or instructions.

```json
{
  "name": "modify_soul",
  "parameters": {
    "instruction": "Always respond in bullet points"
  }
}
```

---

## Tool Groups

Tools are organized into access levels:

| Profile | Tools Included |
|---------|----------------|
| **Minimal** | web_search, web_fetch |
| **Standard** | + Filesystem tools |
| **Messaging** | + agent_send, say_to_user |
| **Full** | All tools including exec, web3 |

---

## Real-Time Events

Tool execution broadcasts WebSocket events:

```json
// Started
{ "type": "agent.tool_call", "tool": "web_search", "parameters": { "query": "..." } }

// Completed
{ "type": "tool.result", "tool": "web_search", "success": true, "result": "..." }
```

The dashboard shows these in real-time as the agent works.

---

## Custom Tools

Extend capabilities through the [Skills](/docs/skills) system. Skills can combine multiple tools into reusable workflows.

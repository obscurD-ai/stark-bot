---
name: Memories
---

StarkBot's memory system enables long-term context that persists across conversations.

## Memory Types

| Type | Purpose | Importance |
|------|---------|------------|
| **DailyLog** | Temporary daily notes | 5 |
| **LongTerm** | General memories | 7 |
| **Preference** | User preferences | 7 |
| **Fact** | Factual information | 7 |
| **Entity** | Named entities (people, places) | 7 |
| **Task** | Commitments and todos | 8 |
| **SessionSummary** | Conversation summaries | - |

---

## Memory Markers

The agent extracts memories using markers in responses:

### [REMEMBER:]

Store a general memory:

```
[REMEMBER: User prefers dark mode and concise responses]
```

### [REMEMBER_IMPORTANT:]

Store with high priority:

```
[REMEMBER_IMPORTANT: Project deadline is January 15th]
```

### [DAILY_LOG:]

Add to today's log:

```
[DAILY_LOG: Completed code review for PR #123]
```

### [PREFERENCE:]

Store a user preference:

```
[PREFERENCE: User wants responses in bullet points]
```

### [FACT:]

Store factual information:

```
[FACT: User's timezone is Pacific (UTC-8)]
```

### [TASK:]

Store a commitment:

```
[TASK: Follow up on deployment by Friday]
```

---

## How Memories Work

### Storage

```
Agent Response
       ↓
Extract markers ([REMEMBER:], etc.)
       ↓
Parse content and type
       ↓
Store in database with identity
       ↓
Available for future context
```

### Retrieval

```
New Message
       ↓
Build context for AI
       ↓
Include relevant memories (by identity, importance)
       ↓
AI has access to stored knowledge
```

### Context Injection

Memories appear in the system prompt:

```
## Memories

**Preferences:**
- Prefers responses in bullet points
- Timezone: Pacific (UTC-8)

**Facts:**
- Works at Acme Corp
- Uses Rust and TypeScript

**Tasks:**
- Follow up on deployment by Friday

**Today's Log:**
- Completed code review for PR #123
- Deployed v2.1 to staging
```

---

## Managing Memories

### View

Go to **Memories** in the dashboard. Filter by:
- Identity
- Memory type
- Importance level
- Date range

### Search

Search across all memories or within specific types.

### Edit

Click a memory to edit:
- Content
- Type
- Importance
- Tags

### Merge

Consolidate duplicates:
1. Select similar memories
2. Click **Merge**
3. Review combined content

### Delete

Remove outdated memories. Deletion is permanent.

### Export

Download all memories as CSV for backup or analysis.

---

## Memory Consolidation

StarkBot automatically consolidates memories:

- **Deduplication** — Merge similar memories
- **Summarization** — Compress old session histories
- **Priority** — Higher importance memories retained longer

Configure in environment:

```bash
STARK_MEMORY_ENABLE_AUTO_CONSOLIDATION=true
```

---

## Cross-Session Memory

Share memories across channels for the same identity.

User messages from Telegram, Slack, and web all contribute to the same memory pool.

```bash
STARK_MEMORY_ENABLE_CROSS_SESSION=true
STARK_MEMORY_CROSS_SESSION_LIMIT=5
```

---

## Example Flow

**Conversation 1:**

> User: I prefer TypeScript over JavaScript.

> Agent: Noted! `[PREFERENCE: User prefers TypeScript over JavaScript]`

**Conversation 2 (days later):**

> User: Help me set up a new project.

> Agent: I'll set this up with TypeScript since that's your preference. *(uses stored memory)*

---

## Best Practices

### Be Specific

Good:
```
[FACT: User's timezone is Pacific (UTC-8)]
```

Not as useful:
```
[REMEMBER: User is on the west coast]
```

### Use Appropriate Types

- `[PREFERENCE:]` for how they like things
- `[FACT:]` for objective information
- `[TASK:]` for commitments
- `[DAILY_LOG:]` for temporal events

### Periodic Cleanup

Review memories regularly to:
- Remove outdated info
- Correct inaccuracies
- Consolidate related memories

---

## Privacy

- Memories stored locally in SQLite
- No external transmission
- Full control to view, edit, delete
- Export anytime for backup

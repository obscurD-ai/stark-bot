# Plan: Special Roles (Enriched Safe Mode)

## Context

Currently, safe mode is absolute — all non-admin users get the same restricted tool set (`SAFE_MODE_ALLOW_LIST` + Web group). The goal is to create "colored safe mode" where specific users can be granted additional tools/skills via named **special roles**, giving them a privileged safe-mode experience while keeping untrusted users restricted.

**Data flow**: Admin creates a special role (e.g. "power_user" with extra tools like `twitter_post`), then assigns it to a specific user+platform combo. When that user opens a safe-mode channel, the dispatcher enriches the safe mode `ToolConfig.allow_list` with the role's extra tools.

---

## Step 1: Model Structs

**New file**: `stark-backend/src/models/special_role.rs`

- `SpecialRole` — `id`, `name` (unique), `allowed_tools: Vec<String>`, `allowed_skills: Vec<String>`, `description`, timestamps
- `SpecialRoleAssignment` — `id`, `channel_type`, `user_id`, `special_role_name`, `created_at`
- `SpecialRoleGrants` — merged result struct with `extra_tools` and `extra_skills` (used by dispatcher)

**Modify**: `stark-backend/src/models/mod.rs` — add `pub mod special_role` + re-exports

---

## Step 2: Database Schema

**File**: `stark-backend/src/db/sqlite.rs` (before `Ok(())` at line ~1561)

Two new tables:

```sql
CREATE TABLE IF NOT EXISTS special_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    allowed_tools TEXT NOT NULL DEFAULT '[]',   -- JSON array
    allowed_skills TEXT NOT NULL DEFAULT '[]',  -- JSON array
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS special_role_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_type TEXT NOT NULL,                  -- discord, twitter, telegram, etc.
    user_id TEXT NOT NULL,                       -- platform-specific user ID
    special_role_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (special_role_name) REFERENCES special_roles(name) ON DELETE CASCADE,
    UNIQUE(channel_type, user_id, special_role_name)
);

CREATE INDEX IF NOT EXISTS idx_sra_lookup ON special_role_assignments(channel_type, user_id);
```

The index on `(channel_type, user_id)` optimizes the dispatcher's hot-path lookup.

---

## Step 3: Database Operations

**New file**: `stark-backend/src/db/tables/special_roles.rs`

`impl Database` with:
- `list_special_roles()` / `get_special_role(name)` / `upsert_special_role(role)` / `delete_special_role(name)`
- `list_special_role_assignments(role_name: Option)` / `create_special_role_assignment(a)` / `delete_special_role_assignment(id)` / `delete_special_role_assignment_by_key(channel_type, user_id, role_name)`
- `get_special_role_grants(channel_type, user_id) -> SpecialRoleGrants` — JOINs assignments+roles, merges all granted tools/skills for the user. This is the hot-path function called by the dispatcher.

**Modify**: `stark-backend/src/db/tables/mod.rs` — add `pub mod special_roles;`

---

## Step 4: Dispatcher Integration (Core Change)

**File**: `stark-backend/src/channels/dispatcher.rs` (lines 722-736)

After `tool_config = crate::tools::ToolConfig::safe_mode();` at line 735, add special role enrichment:

```rust
// After line 735: tool_config = crate::tools::ToolConfig::safe_mode();

// Check for special role grants that enrich safe mode for this user
match self.db.get_special_role_grants(&message.channel_type, &message.user_id) {
    Ok(grants) if !grants.is_empty() => {
        log::info!(
            "[DISPATCH] Special role enrichment for user {} on {}: +tools={:?}",
            message.user_id, message.channel_type, grants.extra_tools
        );
        for tool_name in &grants.extra_tools {
            if !tool_config.allow_list.contains(tool_name) {
                tool_config.allow_list.push(tool_name.clone());
            }
        }
    }
    Ok(_) => {} // No special role
    Err(e) => log::warn!("[DISPATCH] Failed to check special role grants: {}", e),
}
```

Also update `build_system_prompt` (line ~3946) to make the safe-mode tool list dynamic:
- Change `_tool_config` parameter to `tool_config`
- Replace the hard-coded tool list (lines 3960-3967) with a dynamic list from `tool_config.allow_list`

---

## Step 5: Backend API Controller

**New file**: `stark-backend/src/controllers/special_roles.rs`

Following the `controllers/agent_subtypes.rs` pattern:

| Method | Route | Handler |
|--------|-------|---------|
| GET | `/api/special-roles` | list_roles |
| POST | `/api/special-roles` | create_role |
| GET | `/api/special-roles/{name}` | get_role |
| PUT | `/api/special-roles/{name}` | update_role |
| DELETE | `/api/special-roles/{name}` | delete_role |
| GET | `/api/special-roles/assignments` | list_assignments (opt `?role_name=`) |
| POST | `/api/special-roles/assignments` | create_assignment |
| DELETE | `/api/special-roles/assignments/{id}` | delete_assignment |

Uses standard `validate_session_from_request()` for auth.

**Modify**: `stark-backend/src/controllers/mod.rs` — add `pub mod special_roles;`
**Modify**: `stark-backend/src/main.rs` — add `.configure(controllers::special_roles::config)`

---

## Step 6: Meta Tool (`modify_special_role`)

**New file**: `stark-backend/src/tools/builtin/core/modify_special_role.rs`

Following the `manage_gateway_channels.rs` pattern. Actions:
- `list_roles` / `create_role` / `delete_role` / `list_assignments` / `assign_role` / `unassign_role`

Properties: `action`, `role_name`, `allowed_tools` (array), `allowed_skills` (array), `description`, `channel_type`, `user_id`

- `group: ToolGroup::System` — available in standard mode only, NOT in safe mode
- `hidden: false`
- Uses `context.database` for DB access

**Modify**: `stark-backend/src/tools/builtin/core/mod.rs` — add module + pub use
**Modify**: `stark-backend/src/tools/mod.rs` — add `registry.register(Arc::new(builtin::ModifySpecialRoleTool::new()));`

---

## Step 7: Frontend API Functions

**File**: `stark-frontend/src/lib/api.ts`

Add interfaces `SpecialRoleInfo` and `SpecialRoleAssignmentInfo`, plus fetch functions:
- `getSpecialRoles()` / `getSpecialRole(name)` / `createSpecialRole(role)` / `updateSpecialRole(name, update)` / `deleteSpecialRole(name)`
- `getSpecialRoleAssignments(roleName?)` / `createSpecialRoleAssignment(a)` / `deleteSpecialRoleAssignment(id)`

---

## Step 8: Frontend Page

**New file**: `stark-frontend/src/pages/SpecialRoles.tsx`

Following the `AgentSubtypes.tsx` pattern — two tabs:

**Tab 1: Roles** — Left list of role names, right panel with edit form:
- Name (text, unique)
- Description (text)
- Allowed Tools (comma-separated or tag input)
- Allowed Skills (comma-separated or tag input)
- Create / Save / Delete buttons

**Tab 2: Assignments** — Table showing all assignments:
- Columns: Channel Type, User ID, Role Name, Actions
- Add form: channel_type dropdown (discord/twitter/telegram/slack/external_channel), user_id text, role name dropdown
- Delete button per row

**Modify**: `stark-frontend/src/App.tsx` — add route `<Route path="/special-roles" element={<SpecialRoles />} />`
**Modify**: `stark-frontend/src/components/layout/Sidebar.tsx` — add NavItem in Developer section: `<NavItem to="/special-roles" icon={ShieldCheck} label="Special Roles" />`
**Modify**: `stark-frontend/src/components/layout/MobileNavDrawer.tsx` — add to devItems array

---

## Implementation Order

1. Models (Step 1)
2. DB schema (Step 2) + DB operations (Step 3)
3. Dispatcher integration (Step 4) + Meta tool (Step 6) + Controller (Step 5) — can parallelize
4. Frontend API (Step 7) + Page (Step 8)

---

## Verification

1. **Build**: `cargo build` — ensure all new files compile
2. **Start**: Run the backend, check logs for table creation
3. **Admin UI**: Navigate to `/special-roles`, create a role with `allowed_tools: ["twitter_post"]`, create an assignment for a discord user_id
4. **Test enrichment**: Send a message as that discord user (non-admin, safe mode). Check dispatcher logs for "Special role enrichment" and verify the user can call `twitter_post`
5. **Test no regression**: Send a message as a different non-admin user — verify they get vanilla safe mode (no enrichment)
6. **Meta tool**: In an admin chat session, call `modify_special_role` with `action: "list_roles"` — verify it returns the created roles
7. **Cascade delete**: Delete a role via UI, verify its assignments are also removed

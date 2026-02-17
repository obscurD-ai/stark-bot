# Super Router Register Upgrade

## Context

When starkbot generates images via the x402-super-router, the AI model has to relay the returned URL (containing a 64-char SHA256 hash) to the user. The model frequently corrupts these long hex strings, producing broken URLs (e.g., 61 chars instead of 64). The fix: cache the URL in a Rust-side register, and have the model reference it with a `{{x402_result.url}}` template token that gets expanded in `say_to_user` before reaching the user.

## Files to Modify

| File | Change |
|------|--------|
| `stark-backend/src/tools/register.rs` | Add `expand_templates()` method + `value_to_display_string()` helper + unit tests |
| `stark-backend/src/tools/builtin/cryptocurrency/x402_post.rs` | Cache JSON response in `x402_result` register at both success paths |
| `stark-backend/src/tools/builtin/core/say_to_user.rs` | Expand `{{register.field}}` templates before returning message |
| `skills/super_router.md` | Instruct model to use `{{x402_result.url}}` instead of retyping URLs |

## Step 1: Add `expand_templates()` to RegisterStore

**File:** `stark-backend/src/tools/register.rs`

Add `regex` and `once_cell` imports. Add method to `impl RegisterStore`:

```rust
pub fn expand_templates(&self, text: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)\}\}")
            .expect("invalid template regex")
    });

    if !text.contains("{{") {
        return text.to_string(); // Fast path
    }

    RE.replace_all(text, |caps: &regex::Captures| {
        let full_ref = &caps[1];
        let (reg_name, field_path) = match full_ref.find('.') {
            Some(dot_pos) => (&full_ref[..dot_pos], Some(&full_ref[dot_pos + 1..])),
            None => (full_ref, None),
        };
        let value = match field_path {
            Some(field) => self.get_field(reg_name, field),
            None => self.get(reg_name),
        };
        match value {
            Some(val) => value_to_display_string(&val),
            None => {
                log::warn!("[REGISTER] Template '{{{{{}}}}}' not resolved", full_ref);
                format!("{{{{{}}}}}", full_ref) // Leave as-is
            }
        }
    }).into_owned()
}
```

Add helper outside impl block:

```rust
fn value_to_display_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => val.to_string(),
    }
}
```

## Step 2: x402_post caches result in register

**File:** `stark-backend/src/tools/builtin/cryptocurrency/x402_post.rs`

Two success paths need caching:

**Path A — Non-402 success (around line 294):**
```rust
if let Ok(json_val) = serde_json::from_str::<Value>(&response_body) {
    context.set_register("x402_result", json_val.clone(), "x402_post");
    // ... existing return
}
```

**Path B — Post-payment success (around line 414):**
```rust
let result_content = if let Ok(json_val) = serde_json::from_str::<Value>(&paid_body) {
    context.set_register("x402_result", json_val.clone(), "x402_post");
    // ... existing format
} else { paid_body };
```

## Step 3: say_to_user expands templates

**File:** `stark-backend/src/tools/builtin/core/say_to_user.rs`

Change `_context` to `context`, add expansion:

```rust
async fn execute(&self, params: Value, context: &ToolContext) -> ToolResult {
    // ...
    let message = context.registers.expand_templates(&params.message);
    let mut result = ToolResult::success(message);
    // ...
}
```

## Step 4: Update super_router.md skill

Tell the model to use `{{x402_result.url}}` instead of retyping URLs. Add to Response Format section:

> **IMPORTANT**: The x402_post response is automatically cached in the `x402_result` register. When sharing the URL with the user, you MUST use `{{x402_result.url}}` instead of retyping the URL. This ensures the URL is transmitted exactly without corruption.

Update guideline #3:

> **Share the URL using the register template**: Use `{{x402_result.url}}` in your say_to_user message. NEVER manually retype or copy the URL.

## Edge Cases

- **Multiple x402_post calls**: Each overwrites the register. Model should present URL before next call.
- **Non-JSON responses**: Register not set; template passes through unchanged.
- **Normal `{{text}}`**: Strict regex only matches identifier patterns; unmatched templates pass through.
- **Performance**: `once_cell::Lazy` regex compiled once; `contains("{{")` fast-path for most messages.

## Verification

1. `cargo test` — new unit tests in register.rs
2. `cargo build` — all files compile
3. Manual test: ask bot to generate an image, verify URL is correct and uncorrupted

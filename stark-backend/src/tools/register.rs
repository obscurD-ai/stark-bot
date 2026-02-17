//! Register Store for passing data between tools safely
//!
//! This module provides a CPU-like "register" system where tool outputs can be
//! cached and later retrieved by other tools. This prevents hallucination of
//! critical data (like transaction parameters) by ensuring data flows directly
//! between tools without passing through the agent's reasoning.
//!
//! # Example
//!
//! ```ignore
//! // Tool 1 caches its output
//! context.registers.set("swap_quote", json!({
//!     "to": "0x...",
//!     "data": "0x...",
//!     "value": "1000000000000000"
//! }));
//!
//! // Tool 2 reads from the register
//! let quote = context.registers.get("swap_quote")?;
//! let to = quote.get("to").unwrap();
//! ```

use ethers::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A monad for tool parameters that can either use a preset (reading from registers)
/// or custom raw parameters provided by the agent.
///
/// This enforces mutual exclusivity at the type level - you cannot accidentally
/// mix preset and custom parameters.
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct MyToolParams {
///     #[serde(flatten)]
///     mode: PresetOrCustom<CustomParams>,
///     network: String,
/// }
///
/// #[derive(Deserialize)]
/// struct CustomParams {
///     url: String,
///     method: String,
/// }
/// ```
#[derive(Debug, Clone)]
pub enum PresetOrCustom<T> {
    /// Use a named preset that reads values from registers
    Preset(String),
    /// Use custom parameters provided directly
    Custom(T),
}

impl<T> PresetOrCustom<T> {
    /// Returns true if this is a preset
    pub fn is_preset(&self) -> bool {
        matches!(self, PresetOrCustom::Preset(_))
    }

    /// Returns the preset name if this is a preset
    pub fn preset_name(&self) -> Option<&str> {
        match self {
            PresetOrCustom::Preset(name) => Some(name),
            PresetOrCustom::Custom(_) => None,
        }
    }

    /// Returns the custom value if this is custom
    pub fn custom(&self) -> Option<&T> {
        match self {
            PresetOrCustom::Preset(_) => None,
            PresetOrCustom::Custom(v) => Some(v),
        }
    }

    /// Consume and return the custom value if this is custom
    pub fn into_custom(self) -> Option<T> {
        match self {
            PresetOrCustom::Preset(_) => None,
            PresetOrCustom::Custom(v) => Some(v),
        }
    }
}

/// Custom deserializer for PresetOrCustom
///
/// If "preset" field exists, returns Preset variant.
/// Otherwise, attempts to deserialize T from the remaining fields.
impl<'de, T: Deserialize<'de>> Deserialize<'de> for PresetOrCustom<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize as a generic Value to inspect
        let value = Value::deserialize(deserializer)?;

        // Check if "preset" field exists
        if let Some(preset) = value.get("preset").and_then(|v| v.as_str()) {
            return Ok(PresetOrCustom::Preset(preset.to_string()));
        }

        // Otherwise, try to deserialize as T
        T::deserialize(value)
            .map(PresetOrCustom::Custom)
            .map_err(serde::de::Error::custom)
    }
}

/// Intrinsic registers that are lazily computed when accessed.
/// These are always available without needing explicit tool calls.
pub enum IntrinsicRegister {
    WalletAddress,
}

impl IntrinsicRegister {
    /// Match a register name to an intrinsic
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "wallet_address" => Some(Self::WalletAddress),
            _ => None,
        }
    }

    /// Resolve the intrinsic value
    pub fn resolve(&self) -> Option<Value> {
        match self {
            Self::WalletAddress => {
                let pk = crate::config::burner_wallet_private_key()?;
                let wallet: LocalWallet = pk.parse().ok()?;
                Some(json!(format!("{:?}", wallet.address())))
            }
        }
    }
}

/// Session-scoped register store for passing data between tools
/// without flowing through the agent's reasoning.
///
/// This is critical for financial transactions where data integrity
/// must be preserved (e.g., swap calldata from 0x quotes).
#[derive(Debug, Clone, Default)]
pub struct RegisterStore {
    inner: Arc<RwLock<HashMap<String, RegisterEntry>>>,
}

/// A single register entry with metadata
#[derive(Debug, Clone)]
pub struct RegisterEntry {
    /// The stored value
    pub value: Value,
    /// Source tool that created this entry
    pub source_tool: String,
    /// Timestamp when the entry was created
    pub created_at: std::time::Instant,
}

impl RegisterStore {
    /// Create a new empty register store
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a value in the register
    ///
    /// # Arguments
    /// * `key` - The register name (e.g., "swap_quote", "gas_price")
    /// * `value` - The JSON value to store
    /// * `source_tool` - Name of the tool that created this entry
    pub fn set(&self, key: &str, value: Value, source_tool: &str) {
        if let Ok(mut store) = self.inner.write() {
            log::info!(
                "[REGISTER] Set '{}' from tool '{}' (keys: {:?})",
                key,
                source_tool,
                value.as_object().map(|o| o.keys().collect::<Vec<_>>())
            );
            store.insert(
                key.to_string(),
                RegisterEntry {
                    value,
                    source_tool: source_tool.to_string(),
                    created_at: std::time::Instant::now(),
                },
            );
        }
    }

    /// Get a value from the register
    ///
    /// Returns None if the key doesn't exist.
    /// Falls back to intrinsic resolution for special registers like `wallet_address`.
    pub fn get(&self, key: &str) -> Option<Value> {
        // First check explicit registers
        if let Some(entry) = self.get_entry(key) {
            return Some(entry.value);
        }

        // Fall back to intrinsic resolution
        IntrinsicRegister::from_name(key).and_then(|i| i.resolve())
    }

    /// Get the full entry (value + metadata) from the register
    pub fn get_entry(&self, key: &str) -> Option<RegisterEntry> {
        self.inner.read().ok()?.get(key).cloned()
    }

    /// Get entry with metadata, falling back to intrinsic if not set
    pub fn get_entry_or_intrinsic(&self, key: &str) -> Option<RegisterEntry> {
        // Check explicit first
        if let Some(entry) = self.get_entry(key) {
            return Some(entry);
        }

        // Fall back to intrinsic
        IntrinsicRegister::from_name(key).and_then(|i| {
            i.resolve().map(|value| RegisterEntry {
                value,
                source_tool: "intrinsic".to_string(),
                created_at: std::time::Instant::now(),
            })
        })
    }

    /// Get a specific field from a register value
    ///
    /// # Arguments
    /// * `key` - The register name
    /// * `field` - The field path (e.g., "to", "transaction.data")
    pub fn get_field(&self, key: &str, field: &str) -> Option<Value> {
        let value = self.get(key)?;

        // Handle nested field paths (e.g., "transaction.data")
        let mut current = &value;
        for part in field.split('.') {
            current = current.get(part)?;
        }

        Some(current.clone())
    }

    /// Check if a register exists
    pub fn exists(&self, key: &str) -> bool {
        self.inner
            .read()
            .ok()
            .map(|s| s.contains_key(key))
            .unwrap_or(false)
    }

    /// Clear all registers (at end of execution)
    pub fn clear(&self) {
        if let Ok(mut store) = self.inner.write() {
            log::info!("[REGISTER] Clearing all registers");
            store.clear();
        }
    }

    /// Remove a specific register
    pub fn remove(&self, key: &str) -> Option<Value> {
        self.inner
            .write()
            .ok()
            .and_then(|mut s| s.remove(key))
            .map(|e| e.value)
    }

    /// List all register keys (for debugging)
    pub fn keys(&self) -> Vec<String> {
        self.inner
            .read()
            .ok()
            .map(|s| s.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get age of a register entry in seconds
    pub fn age_secs(&self, key: &str) -> Option<u64> {
        self.get_entry(key)
            .map(|e| e.created_at.elapsed().as_secs())
    }

    /// Check if a register is stale (older than max_age_secs)
    pub fn is_stale(&self, key: &str, max_age_secs: u64) -> bool {
        self.age_secs(key)
            .map(|age| age > max_age_secs)
            .unwrap_or(true)
    }

    /// Expand `{{register_name}}` and `{{register_name.field}}` templates in text.
    ///
    /// - Simple refs like `{{x402_result}}` use `get()` and display the whole value
    /// - Dotted refs like `{{x402_result.url}}` use `get_field()`
    /// - Missing registers are left as-is with a log warning
    pub fn expand_templates(&self, text: &str) -> String {
        // Fast path: skip regex if no templates
        if !text.contains("{{") {
            return text.to_string();
        }

        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)\}\}")
                .expect("valid template regex")
        });

        RE.replace_all(text, |caps: &regex::Captures| {
            let full_ref = &caps[1];

            let resolved = if let Some(dot_pos) = full_ref.find('.') {
                // Dotted path: "register_name.field.subfield"
                let key = &full_ref[..dot_pos];
                let field = &full_ref[dot_pos + 1..];
                self.get_field(key, field)
            } else {
                // Simple ref: "register_name"
                self.get(full_ref)
            };

            match resolved {
                Some(value) => value_to_display_string(&value),
                None => {
                    log::warn!(
                        "[REGISTER] Template ref '{{{{{}}}}}' not found, leaving as-is",
                        full_ref
                    );
                    caps[0].to_string()
                }
            }
        })
        .into_owned()
    }
}

/// Convert a JSON value to a display string:
/// - Strings are unwrapped (no quotes)
/// - Everything else is compact JSON
fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_register_set_get() {
        let store = RegisterStore::new();

        store.set(
            "test_key",
            json!({"to": "0x123", "value": "1000"}),
            "test_tool",
        );

        let value = store.get("test_key").unwrap();
        assert_eq!(value.get("to").unwrap(), "0x123");
        assert_eq!(value.get("value").unwrap(), "1000");
    }

    #[test]
    fn test_register_get_field() {
        let store = RegisterStore::new();

        store.set(
            "quote",
            json!({
                "transaction": {
                    "to": "0xabc",
                    "data": "0x1234"
                },
                "buyAmount": "5000"
            }),
            "x402_preset_fetch",
        );

        assert_eq!(
            store.get_field("quote", "transaction.to").unwrap(),
            json!("0xabc")
        );
        assert_eq!(
            store.get_field("quote", "transaction.data").unwrap(),
            json!("0x1234")
        );
        assert_eq!(
            store.get_field("quote", "buyAmount").unwrap(),
            json!("5000")
        );
    }

    #[test]
    fn test_register_clear() {
        let store = RegisterStore::new();

        store.set("key1", json!("value1"), "tool1");
        store.set("key2", json!("value2"), "tool2");

        assert!(store.exists("key1"));
        assert!(store.exists("key2"));

        store.clear();

        assert!(!store.exists("key1"));
        assert!(!store.exists("key2"));
    }

    #[test]
    fn test_register_clone_shares_state() {
        let store1 = RegisterStore::new();
        let store2 = store1.clone();

        store1.set("shared", json!("data"), "tool1");

        // store2 should see the data set by store1
        assert_eq!(store2.get("shared").unwrap(), json!("data"));
    }

    #[test]
    fn test_register_entry_metadata() {
        let store = RegisterStore::new();

        store.set("test", json!({"key": "value"}), "my_tool");

        let entry = store.get_entry("test").unwrap();
        assert_eq!(entry.source_tool, "my_tool");
        assert!(entry.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_expand_templates_simple() {
        let store = RegisterStore::new();
        store.set("greeting", json!("hello world"), "test");

        assert_eq!(
            store.expand_templates("Say: {{greeting}}"),
            "Say: hello world"
        );
    }

    #[test]
    fn test_expand_templates_dotted() {
        let store = RegisterStore::new();
        store.set(
            "x402_result",
            json!({"url": "https://cdn.example.com/image.png", "prompt": "a cat"}),
            "x402_post",
        );

        assert_eq!(
            store.expand_templates("Here is the image: {{x402_result.url}}"),
            "Here is the image: https://cdn.example.com/image.png"
        );
    }

    #[test]
    fn test_expand_templates_missing_left_as_is() {
        let store = RegisterStore::new();

        let text = "Missing: {{nonexistent.field}}";
        assert_eq!(store.expand_templates(text), text);
    }

    #[test]
    fn test_expand_templates_no_templates_fast_path() {
        let store = RegisterStore::new();

        let text = "No templates here";
        assert_eq!(store.expand_templates(text), text);
    }

    #[test]
    fn test_expand_templates_multiple() {
        let store = RegisterStore::new();
        store.set(
            "result",
            json!({"url": "https://example.com/img.png", "type": "image"}),
            "test",
        );

        assert_eq!(
            store.expand_templates("Type: {{result.type}}, URL: {{result.url}}"),
            "Type: image, URL: https://example.com/img.png"
        );
    }

    #[test]
    fn test_expand_templates_non_string_value() {
        let store = RegisterStore::new();
        store.set("data", json!({"count": 42, "active": true}), "test");

        assert_eq!(store.expand_templates("Count: {{data.count}}"), "Count: 42");
        assert_eq!(
            store.expand_templates("Active: {{data.active}}"),
            "Active: true"
        );
    }

    #[test]
    fn test_expand_templates_whole_object() {
        let store = RegisterStore::new();
        store.set("simple", json!({"a": 1}), "test");

        // Whole object reference should produce compact JSON
        let result = store.expand_templates("Data: {{simple}}");
        assert!(result.contains("\"a\":1") || result.contains("\"a\": 1"));
    }

    #[test]
    fn test_value_to_display_string() {
        assert_eq!(value_to_display_string(&json!("hello")), "hello");
        assert_eq!(value_to_display_string(&json!(42)), "42");
        assert_eq!(value_to_display_string(&json!(true)), "true");
        assert_eq!(value_to_display_string(&json!(null)), "null");
    }
}

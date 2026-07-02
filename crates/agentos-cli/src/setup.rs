//! Wires agentos into an agent's configuration. Dry-run by default
//! (security finding 5): show the exact file and content first, write only
//! with --apply. Hook commands use the absolute exe path (no PATH hijack)
//! and live in .claude/settings.local.json because that path is
//! machine-specific and should not be committed.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn claude_code(project_root: &Path, apply: bool) -> Result<()> {
    let exe = std::env::current_exe().context("resolving agentos executable path")?;
    let exe = exe.to_string_lossy();
    let settings_path = project_root.join(".claude").join("settings.local.json");

    let mut settings: Value = if settings_path.exists() {
        serde_json::from_str(&fs::read_to_string(&settings_path)?).with_context(|| {
            format!(
                "{} exists but is not valid JSON — fix it by hand first",
                settings_path.display()
            )
        })?
    } else {
        json!({})
    };

    let mut changed = false;
    for (event, cmd) in [
        ("Stop", format!("\"{exe}\" hook stop")),
        ("UserPromptSubmit", format!("\"{exe}\" hook prompt")),
    ] {
        let root = settings
            .as_object_mut()
            .context("settings root must be a JSON object")?;
        let events = root
            .entry("hooks")
            .or_insert(json!({}))
            .as_object_mut()
            .context("\"hooks\" must be a JSON object")?;
        let groups = events
            .entry(event)
            .or_insert(json!([]))
            .as_array_mut()
            .with_context(|| format!("hooks.{event} must be a JSON array"))?;
        // An entry is ours if it runs `agentos hook <event>`, whatever the
        // binary path — update stale paths in place instead of duplicating.
        let marker = format!("hook {}", if event == "Stop" { "stop" } else { "prompt" });
        let mut found = false;
        for g in groups.iter_mut() {
            let Some(hs) = g.get_mut("hooks").and_then(Value::as_array_mut) else {
                continue;
            };
            for h in hs.iter_mut() {
                let existing = h["command"].as_str().unwrap_or("");
                if existing.contains("agentos") && existing.trim_end().ends_with(&marker) {
                    found = true;
                    if existing != cmd {
                        h["command"] = json!(cmd);
                        changed = true;
                    }
                }
            }
        }
        if !found {
            groups.push(json!({ "hooks": [{ "type": "command", "command": cmd }] }));
            changed = true;
        }
    }

    let rendered = serde_json::to_string_pretty(&settings)?;
    if !apply {
        println!("dry run — would write {}:", settings_path.display());
        println!("{rendered}");
        println!("\nnothing written. run `agentos setup claude-code --apply` to apply.");
        return Ok(());
    }
    if !changed {
        println!("already configured — nothing to change");
        return Ok(());
    }
    if let Some(dir) = settings_path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&settings_path, rendered)?;
    println!("wrote {}", settings_path.display());
    println!("restart Claude Code (or start a new session) to pick up the hooks");
    Ok(())
}

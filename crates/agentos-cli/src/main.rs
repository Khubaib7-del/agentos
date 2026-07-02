mod hooks;
mod setup;

use agentos_core::Store;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(
    name = "agentos",
    version,
    about = "Companion layer for AI coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the .agentos state directory in the current project
    Init,
    /// Record a project decision
    Decide {
        /// The decision itself, e.g. "DB: PostgreSQL"
        text: String,
        /// Rationale stored alongside the decision
        #[arg(long)]
        why: Option<String>,
        /// Lock it — agents will be warned on conflicting proposals
        #[arg(long)]
        lock: bool,
    },
    /// Queue a review note for the agent without interrupting it
    Note {
        /// The idea to deliver when the agent finishes its current task
        text: String,
    },
    /// Show recorded decisions and pending review notes
    List {
        /// Output as JSON instead of plain text
        #[arg(long)]
        json: bool,
    },
    /// Agent hook entry points (called by the agent, not by hand)
    #[command(subcommand)]
    Hook(HookEvent),
    /// Run the MCP stdio server (spawned by agents, not by hand)
    Mcp,
    /// Wire agentos into an agent's configuration (dry run unless --apply)
    #[command(subcommand)]
    Setup(SetupTarget),
}

#[derive(Subcommand)]
enum HookEvent {
    /// Claude Code Stop hook: deliver queued review notes
    Stop,
    /// Claude Code UserPromptSubmit hook: inject locked decisions
    Prompt,
}

#[derive(Subcommand)]
enum SetupTarget {
    /// Configure Claude Code hooks in .claude/settings.local.json
    ClaudeCode {
        /// Actually write the file (default is a dry run)
        #[arg(long)]
        apply: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = env::current_dir()?;

    match cli.command {
        Command::Init => {
            Store::init(&cwd)?;
            println!("initialized .agentos in {}", cwd.display());
        }
        Command::Decide { text, why, lock } => {
            let store = Store::open(&cwd)?;
            let d = store.add_decision(&text, why.as_deref(), lock)?;
            let suffix = if d.locked { " (locked)" } else { "" };
            println!("decision #{} recorded{suffix}", d.id);
        }
        Command::Note { text } => {
            let store = Store::open(&cwd)?;
            let n = store.add_note(&text)?;
            println!("note #{} queued for the agent's next review pass", n.id);
        }
        Command::List { json } => {
            let store = Store::open(&cwd)?;
            let decisions = store.decisions()?;
            let pending = store.pending_notes()?;
            if json {
                let out = serde_json::json!({
                    "decisions": decisions,
                    "pending_notes": pending,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("decisions ({}):", decisions.len());
                for d in &decisions {
                    let lock = if d.locked { " [locked]" } else { "" };
                    println!("  #{} {}{lock}", d.id, d.text);
                }
                println!("pending review notes ({}):", pending.len());
                for n in &pending {
                    println!("  #{} {}", n.id, n.text);
                }
            }
        }
        Command::Hook(HookEvent::Stop) => hooks::run_stop(),
        Command::Hook(HookEvent::Prompt) => hooks::run_prompt(),
        Command::Mcp => agentos_mcp::serve(&cwd)?,
        Command::Setup(SetupTarget::ClaudeCode { apply }) => setup::claude_code(&cwd, apply)?,
    }
    Ok(())
}

use agentos_core::Store;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(name = "agentos", version, about = "Companion layer for AI coding agents")]
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
    List,
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
        Command::List => {
            let store = Store::open(&cwd)?;
            let decisions = store.decisions()?;
            println!("decisions ({}):", decisions.len());
            for d in &decisions {
                let lock = if d.locked { " [locked]" } else { "" };
                println!("  #{} {}{lock}", d.id, d.text);
            }
            let pending = store.pending_notes()?;
            println!("pending review notes ({}):", pending.len());
            for n in &pending {
                println!("  #{} {}", n.id, n.text);
            }
        }
    }
    Ok(())
}

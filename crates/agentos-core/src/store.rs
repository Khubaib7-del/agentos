use crate::model::{Decision, NoteStatus, ReviewNote};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const STATE_DIR: &str = ".agentos";
const DECISIONS_FILE: &str = "decisions.json";
const QUEUE_FILE: &str = "review-queue.json";

/// File-backed state for one project. Plain JSON with a rendered markdown
/// mirror, so the user can always read and version the state without agentos.
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn init(project_root: &Path) -> Result<Self> {
        let root = project_root.join(STATE_DIR);
        if root.exists() {
            bail!("{} already exists — this project is initialized", root.display());
        }
        fs::create_dir_all(&root)
            .with_context(|| format!("creating {}", root.display()))?;
        let store = Self { root };
        store.write_json(DECISIONS_FILE, &Vec::<Decision>::new())?;
        store.write_json(QUEUE_FILE, &Vec::<ReviewNote>::new())?;
        store.render_decisions_md(&[])?;
        Ok(store)
    }

    pub fn open(project_root: &Path) -> Result<Self> {
        let root = project_root.join(STATE_DIR);
        if !root.is_dir() {
            bail!("no {STATE_DIR} directory in {} — run `agentos init` first", project_root.display());
        }
        Ok(Self { root })
    }

    pub fn add_decision(&self, text: &str, why: Option<&str>, locked: bool) -> Result<Decision> {
        let mut all: Vec<Decision> = self.read_json(DECISIONS_FILE)?;
        let decision = Decision {
            id: all.last().map_or(1, |d| d.id + 1),
            text: text.to_string(),
            why: why.map(str::to_string),
            locked,
            made_at: chrono::Utc::now(),
        };
        all.push(decision.clone());
        self.write_json(DECISIONS_FILE, &all)?;
        self.render_decisions_md(&all)?;
        Ok(decision)
    }

    pub fn decisions(&self) -> Result<Vec<Decision>> {
        self.read_json(DECISIONS_FILE)
    }

    pub fn add_note(&self, text: &str) -> Result<ReviewNote> {
        let mut all: Vec<ReviewNote> = self.read_json(QUEUE_FILE)?;
        let note = ReviewNote {
            id: all.last().map_or(1, |n| n.id + 1),
            text: text.to_string(),
            status: NoteStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        all.push(note.clone());
        self.write_json(QUEUE_FILE, &all)?;
        Ok(note)
    }

    pub fn notes(&self) -> Result<Vec<ReviewNote>> {
        self.read_json(QUEUE_FILE)
    }

    pub fn pending_notes(&self) -> Result<Vec<ReviewNote>> {
        Ok(self
            .notes()?
            .into_iter()
            .filter(|n| n.status == NoteStatus::Pending)
            .collect())
    }

    fn read_json<T: serde::de::DeserializeOwned>(&self, file: &str) -> Result<T> {
        let path = self.root.join(file);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
    }

    /// Write via temp file + rename so a crash mid-write never corrupts state.
    fn write_json<T: serde::Serialize>(&self, file: &str, value: &T) -> Result<()> {
        let path = self.root.join(file);
        let tmp = self.root.join(format!("{file}.tmp"));
        fs::write(&tmp, serde_json::to_string_pretty(value)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, &path)
            .with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    fn render_decisions_md(&self, decisions: &[Decision]) -> Result<()> {
        let mut md = String::from(
            "# Decision log\n\nManaged by agentos — record entries with `agentos decide`, not by hand.\n",
        );
        for d in decisions {
            let lock = if d.locked { " [locked]" } else { "" };
            md.push_str(&format!("\n## #{} — {}{}\n", d.id, d.text, lock));
            md.push_str(&format!("*{}*\n", d.made_at.format("%Y-%m-%d %H:%M UTC")));
            if let Some(why) = &d.why {
                md.push_str(&format!("\n**Why:** {why}\n"));
            }
        }
        fs::write(self.root.join("decisions.md"), md)?;
        Ok(())
    }
}

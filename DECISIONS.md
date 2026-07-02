# Decision Log

The project's own decision log — the artifact AgentOS will eventually automate. Append-only; reversals get a new entry, never an edit.

---

## 2026-07-02 — Product: companion layer, not another agent 🔒
Build the layer that improves collaboration with existing coding agents (Claude Code, Cursor, Copilot, Codex, Gemini CLI, Antigravity). We do not generate code and we do not compete with the agents.
**Why:** the "better codegen" space is saturated; the collaboration/continuity space is empty and grows with multi-agent usage.

## 2026-07-02 — Strategy: vertical-first on Claude Code 🔒
V1 goes deep on Claude Code (hooks + MCP + statusline). Other agents get file-based memory + MCP from day one, deep integration later.
**Why:** Claude Code is the only agent exposing hooks/statusline — it's the only place the full UX is implementable now; it's also our own daily driver (dogfooding).

## 2026-07-02 — Architecture: single Rust binary, subcommand per surface 🔒
One `agentos` binary: `mcp`, `hook <event>`, `statusline`, plus user CLI commands. Official `rmcp` SDK for MCP. TypeScript only for the future VS Code panel (thin shell, logic stays in Rust).
**Why:** no runtime for users to install; instant startup matters for hooks; single artifact simplifies signing and distribution.

## 2026-07-02 — Integration: MCP + files are the universal layer, not a VS Code extension 🔒
The cross-agent story is `AGENTS.md`-style files plus a local stdio MCP server registered in each agent's config. No attempt to inject UI into other agents' panels.
**Why:** agents share no UI/API surface; MCP and instruction files are the only integration points they all support.

## 2026-07-02 — State: plain markdown + JSON in `.agentos/`, user-ownable 🔒
No database, no proprietary format. `agentos render` is the only writer of agent-facing files, using marked managed regions.
**Why:** users must be able to read/edit/version memory without our tool (trust + longevity); plain text survives our product dying.

## 2026-07-02 — Privacy: local-first, zero telemetry in v1 🔒
No cloud, no account, no telemetry of any kind in v1. Remote anything (team sync) is post-1.0 and opt-in.
**Why:** we parse users' full agent transcripts; the only acceptable posture is that nothing leaves the machine. Also a differentiator.

## 2026-07-02 — Honesty rule: no faked capabilities
Where an agent doesn't expose data (context % on Cursor/Copilot), we say "not supported" — never estimate-and-pretend. The feasibility matrix in docs/02 is the source of truth for claims.
**Why:** the product's entire value is trust in what it injects and reports.

## 2026-07-02 — Naming: "AgentOS" is internal codename only
Public name chosen before Milestone 3 launch. Candidates: ContextKit, DevContext, AgentCompanion (availability unverified).
**Why:** renaming after launch is expensive; before launch it's free.

🔒 = locked decision (changing it requires a new entry explaining why).

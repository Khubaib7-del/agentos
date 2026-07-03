# AgentOS (working codename)

**A companion layer for AI coding agents.** It doesn't generate code — it improves the collaboration between developers and agents like Claude Code, Cursor, GitHub Copilot, Codex, Gemini CLI, and Antigravity.

> Status: early development (July 2026). Core CLI, Claude Code hooks, and MCP server are working; see Usage below.

## The one-line pitch

Every AI coding agent forgets your decisions, can't be steered mid-task without derailing, and hides its context health. AgentOS is a single local binary that fixes all three — for every agent at once.

## Core features (v1 scope)

| Feature | What it does |
|---|---|
| **Review Queue** | Drop ideas while the agent works; they're delivered as review comments when it finishes — no interruption, no derail |
| **Decision Log & Locks** | Architectural decisions become persistent, injected into every prompt; conflicts get flagged |
| **Context Health** | Live context-window %, degradation estimate, usage-reset timer (Claude Code) |
| **Session Snapshots** | One command captures state (decisions, TODOs, architecture) for the next session or a different agent |
| **Cross-Agent Memory** | File-based memory readable by every agent via `AGENTS.md` conventions + MCP |

## Usage

```
agentos init                          create the .agentos state directory in the current project
agentos decide "DB: PostgreSQL"       record a project decision
        --why "team knows it"         rationale stored alongside the decision
        --lock                        agents get warned on conflicting proposals
agentos note "check error handling"   queue a review note; delivered when the agent finishes its task
agentos list                          show recorded decisions and pending review notes
agentos list --json                   same data as JSON, for scripts (includes why/status/timestamps)
agentos render                        write decisions into AGENTS.md so Cursor/Codex/Copilot see them
agentos context                       context health of the latest Claude Code session in this project
agentos snapshot "summary" --todo t   save a session snapshot (decisions + open notes bundled in)
agentos restore                       print the latest snapshot — paste into any agent to restore context
agentos setup claude-code --apply     wire the Claude Code hooks into .claude/settings.local.json
```

The plain-text `list` output is human-oriented; scripts should use `--json`, whose schema is the stable interface.

### Install as a Claude Code plugin

With the `agentos` binary on PATH (`cargo install --path crates/agentos-cli`), the repo doubles as a plugin marketplace:

```
/plugin marketplace add Khubaib7-del/agentos
/plugin install agentos@agentos
```

This wires the Stop/UserPromptSubmit hooks, registers the MCP server, and adds `/agentos:note`, `/agentos:decide`, and `/agentos:status` slash commands. Manual alternative: `agentos setup claude-code --apply` per project.

## Documentation

- [Vision & Product Definition](docs/01-vision.md) — problem, features, differentiation
- [Architecture](docs/02-architecture.md) — single Rust binary, MCP + hooks + statusline, per-agent feasibility matrix
- [Security & Threat Model](docs/03-security.md) — findings on prompt injection, secrets, supply chain, and how we mitigate them
- [Roadmap](docs/04-roadmap.md) — MVP milestones
- [Distribution & Launch](docs/05-launch-and-distribution.md) — install channels, landing page, docs site, build-in-public, Product Hunt
- [DECISIONS.md](DECISIONS.md) — the project's own decision log (we dogfood our own concept)

## Design principles

1. **Local-first.** No cloud, no telemetry, no account for v1. Everything lives in the repo and on the user's machine.
2. **Agent-agnostic core, agent-specific depth.** Files + MCP work everywhere; Claude Code hooks give the deepest experience first.
3. **Honest capability claims.** If an agent doesn't expose something (e.g. Cursor's context usage), we say so instead of faking it.
4. **The user owns the memory.** Plain markdown + JSON, readable and editable without our tool.

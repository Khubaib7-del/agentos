# Thruline + Command Code

Command Code reads Thruline's memory two ways: the `AGENTS.md` file (automatic) and MCP tools (one-time setup). Steps below take ~3 minutes.

## 1. Install Thruline

```
npm install -g thruline
```

Check it works — **open a new terminal** (PATH updates don't reach already-open ones) and run:

```
thruline --version
```

Expected: `thruline 0.0.3` (or newer). If you get "not recognized", see Troubleshooting below.

## 2. Turn it on in your project

From your project's root folder:

```
thruline init
thruline decide "DB: PostgreSQL" --lock     (example — record a real decision)
thruline render
```

`render` writes your decisions into `AGENTS.md`, which Command Code reads automatically as project memory. Re-run `thruline render` whenever you add decisions.

## 3. Connect the MCP tools

Start Command Code (`cmd`), then type:

```
/mcp
```

In the MCP manager, add a new server:

- **Name:** `thruline`
- **Transport:** stdio
- **Command:** `thruline` (if it errors, use the full path — find it with `where thruline` on Windows or `which thruline` on macOS/Linux)
- **Arguments:** `mcp`

Wait for the green/connected indicator. The tools appear as `mcp__thruline__get_decisions`, `mcp__thruline__log_decision`, and five more.

## 4. Verify

Ask Command Code: *"what are this project's recorded decisions?"* — it should answer with your locked decision, via AGENTS.md or the `get_decisions` tool.

## What works here (honest list)

| Feature | Status |
|---|---|
| Decision memory (AGENTS.md + MCP) | ✅ full |
| Logging decisions from chat (`log_decision`) | ✅ full |
| Conflict checking (`check_conflict`) | ✅ full |
| Snapshots (`save_snapshot` / `get_latest_snapshot`) | ✅ full |
| Review queue (`thruline note` while the agent works) | ⚠️ best-effort — AGENTS.md instructs the agent to check the queue when it finishes; if it forgets, say "check the thruline review queue" |
| Context % / statusline | ❌ Command Code doesn't expose context data to external tools |

(Automatic, enforced review-queue delivery is a Claude Code exclusive — it's the only agent with a hook system. Same for `/thruline:` slash commands: pressing `/` in Command Code will not show Thruline entries — ask the agent to run `thruline note "..."` instead. Full picture: [what works where](what-works-where.md).)

## Troubleshooting

- **`thruline` is not recognized** — the npm global folder isn't on PATH, or the terminal is stale. Open a fresh terminal; on Windows run `npm config get prefix` and make sure that folder is in PATH.
- **Install fails downloading the binary** — the installer fetches from GitHub Releases and verifies a SHA-256 checksum. Behind a proxy/offline it can't. Run `node install.js` inside the package folder later, or build from source: `cargo install thruline`.
- **MCP server shows red/error** — the agent launched with a different PATH than your shell. Use the absolute binary path as the Command; `where thruline` prints it.
- **"no .thruline directory" errors** — you're in a folder where `thruline init` hasn't been run. Run it in the project root.
- **Agent ignores queued notes** — expected occasionally (best-effort on non-Claude agents). Prompt: *"check the thruline review queue and address pending notes."*

Anything else broken? Open an issue: https://github.com/Khubaib7-del/thruline/issues

# Thruline + Cursor, Antigravity, Codex, Gemini CLI, Copilot

Every agent that reads `AGENTS.md` and speaks MCP gets Thruline's memory. The recipe is always the same two steps; only the MCP config location differs.

## Step 1 — memory file (works with zero agent config)

```
npm install -g thruline
cd your-project
thruline init
thruline decide "your first decision" --lock
thruline render
```

`render` writes a managed section into `AGENTS.md`. Cursor, Antigravity (v1.20.3+), Codex, and Copilot read it natively; Gemini CLI reads GEMINI.md — pointing it at AGENTS.md in GEMINI.md works. Re-run `thruline render` after new decisions.

## Step 2 — MCP tools (lets the agent read AND write memory)

Register a stdio MCP server: command `thruline`, argument `mcp`. Where:

| Agent | Where to configure |
|---|---|
| Cursor | `.cursor/mcp.json` in the project (or Settings → MCP) |
| Antigravity IDE / CLI | shared MCP config (see antigravity.google/docs/mcp) |
| Codex CLI | `~/.codex/config.toml` |
| Gemini CLI | `~/.gemini/settings.json` |
| VS Code / Copilot | `.vscode/mcp.json` |

Typical JSON shape where a JSON file is used:

```json
{
  "mcpServers": {
    "thruline": { "command": "thruline", "args": ["mcp"] }
  }
}
```

If the server won't start, replace `"thruline"` with the absolute path (`where thruline` / `which thruline`).

## What to expect (honest)

- ✅ Decisions memory, conflict checks, snapshots, logging decisions from chat — full.
- ⚠️ Review queue (`thruline note`) — best-effort: AGENTS.md tells the agent to check the queue when finishing; remind it if it forgets. Enforced delivery exists only in Claude Code (hooks).
- ❌ Context %/reset timer — these agents don't expose context data. (Claude Code CLI exclusive.)

Full capability matrix: [architecture doc](../02-architecture.md).

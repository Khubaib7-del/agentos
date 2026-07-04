---
description: End-of-session ritual — clear the review queue, snapshot, sync AGENTS.md
allowed-tools: Bash(thruline render:*)
---

Wrap up this session properly, in this order:

1. Call the thruline MCP tool `get_review_queue`. If any notes are pending, address each like a code-review comment now (or explain why not), then mark them resolved with `resolve_review_note`.
2. Call `save_snapshot` with a genuinely useful summary: what was accomplished this session, the current state, and what's next — include unfinished work in `todos`.
3. Run `thruline render` so other agents see the latest decisions.
4. Give the user a 3-line sign-off: what was wrapped, where the snapshot lives, what tomorrow starts with.

Extra context from the user, if any: $ARGUMENTS

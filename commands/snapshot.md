---
description: Save a session snapshot (summary + decisions + open notes) for later restore
allowed-tools: Bash(thruline snapshot:*)
---

Save a thruline session snapshot now. Write a concise but complete summary of this session yourself — what was done, current state, what's next — and include unfinished items as --todo flags:

`thruline snapshot "<your summary>" --todo "<unfinished item>" --todo "<another>"`

If the user provided text, treat it as extra context to include: $ARGUMENTS

Confirm the saved file path in one line. The user can restore it in any future session or agent with `thruline restore`.
